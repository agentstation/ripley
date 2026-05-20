use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use colored::Colorize;

use ripley_core::config;
use ripley_core::db::AdvisoryDb;
use ripley_core::feed::osv::OsvClient;
use ripley_core::lockfile;
use ripley_core::matcher;
use ripley_core::types::Severity;

#[derive(Parser)]
#[command(name = "ripley", about = "Supply chain defense for developers")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan lockfiles against known advisories
    Scan {
        /// Path to scan (defaults to current directory)
        path: Option<PathBuf>,
        /// Output format (table or json)
        #[arg(long, default_value = "table")]
        format: String,
        /// Run deep forensic scan
        #[arg(long)]
        deep: bool,
        /// Generate fix prompts for findings
        #[arg(long)]
        fix: bool,
        /// Skip advisory cache, fetch fresh data
        #[arg(long)]
        no_cache: bool,
    },
    /// Manage the package manager guard
    Guard {
        #[command(subcommand)]
        command: GuardCommands,
    },
    /// Show status of Ripley and monitored projects
    Status,
    /// Manage configuration
    Config {
        /// Print the config file path
        #[arg(long)]
        path: bool,
        /// Print the resolved configuration
        #[arg(long)]
        show: bool,
        /// Create a default config file if none exists
        #[arg(long)]
        init: bool,
    },
}

#[derive(Subcommand)]
enum GuardCommands {
    /// Install package manager shims
    Install,
    /// Remove shims and restore original behavior
    Uninstall,
    /// Show which package managers are intercepted
    Status,
    /// Add a package to the trust list
    Trust {
        /// Package name or scope (e.g., "@tanstack/*")
        package: String,
    },
    /// Remove a package from the trust list
    Untrust {
        /// Package name or scope
        package: String,
    },
    /// Show recent interception log
    Log,
}

fn find_lockfiles(dir: &std::path::Path) -> Vec<PathBuf> {
    let mut lockfiles = Vec::new();
    find_lockfiles_recursive(dir, &mut lockfiles, 0);
    lockfiles
}

fn find_lockfiles_recursive(dir: &std::path::Path, lockfiles: &mut Vec<PathBuf>, depth: usize) {
    if depth > 10 {
        return;
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str())
                && name == "package-lock.json"
            {
                lockfiles.push(path);
            }
        } else if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str())
                && (name == "node_modules" || name.starts_with('.'))
            {
                continue;
            }
            find_lockfiles_recursive(&path, lockfiles, depth + 1);
        }
    }
}

fn severity_colored(severity: &Option<Severity>) -> colored::ColoredString {
    match severity {
        Some(Severity::Critical) => "CRITICAL".red().bold(),
        Some(Severity::High) => "HIGH".red(),
        Some(Severity::Medium) => "MEDIUM".yellow(),
        Some(Severity::Low) => "LOW".blue(),
        None => "UNKNOWN".dimmed(),
    }
}

async fn cmd_scan(
    path: PathBuf,
    format: &str,
    _deep: bool,
    _fix: bool,
    no_cache: bool,
) -> anyhow::Result<ExitCode> {
    let cwd = std::env::current_dir()?;
    let cfg = config::load_config(&cwd)?;
    let cache_dir = ripley_core::dirs::cache_dir()?;
    let data_dir = ripley_core::dirs::data_dir()?;
    let db_path = data_dir.join("advisories.redb");
    let db = AdvisoryDb::open(&db_path)?;

    let lockfiles = find_lockfiles(&path);
    if lockfiles.is_empty() {
        eprintln!("No lockfiles found in {}", path.display());
        return Ok(ExitCode::from(2));
    }

    let mut all_packages = Vec::new();
    let mut all_warnings = Vec::new();
    let mut all_risky = Vec::new();

    for lockfile_path in &lockfiles {
        match lockfile::parse_lockfile(lockfile_path) {
            Ok(parsed) => {
                all_packages.extend(parsed.packages);
                all_warnings.extend(parsed.warnings);
                all_risky.extend(parsed.risky_specs);
            }
            Err(e) => {
                eprintln!("Warning: could not parse {}: {e}", lockfile_path.display());
            }
        }
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let last_poll = db.get_last_poll()?;
    let stale = last_poll
        .map(|lp| now - lp > cfg.feeds.stale_threshold_secs)
        .unwrap_or(true);

    if no_cache || stale {
        let client = match OsvClient::new(cache_dir) {
            Ok(c) => c,
            Err(e) => {
                if last_poll.is_none() {
                    eprintln!(
                        "No cached advisories and network unavailable — \
                         run with network access to populate the cache"
                    );
                    return Ok(ExitCode::from(2));
                }
                eprintln!("Warning: could not create feed client: {e}");
                eprintln!("Using cached advisories (may be stale)");
                return run_matcher_and_output(
                    &db,
                    &all_packages,
                    &all_warnings,
                    &all_risky,
                    &path,
                    format,
                    &cfg,
                );
            }
        };

        let unique_packages: Vec<_> = {
            let mut seen = std::collections::HashSet::new();
            all_packages
                .iter()
                .filter(|p| seen.insert((p.ecosystem, p.name.clone())))
                .map(|p| (p.ecosystem, p.name.clone()))
                .collect()
        };

        if !unique_packages.is_empty() {
            match client.query_batch(&unique_packages).await {
                Ok(advisories) => {
                    let mut by_key: std::collections::HashMap<_, Vec<_>> =
                        std::collections::HashMap::new();
                    for adv in &advisories {
                        by_key
                            .entry((adv.ecosystem, adv.package.clone()))
                            .or_default()
                            .push(adv.clone());
                    }
                    for ((eco, pkg), advs) in &by_key {
                        db.store_advisories(*eco, pkg, advs)?;
                    }
                    db.set_last_poll(now)?;
                }
                Err(e) => {
                    if last_poll.is_none() {
                        eprintln!(
                            "No cached advisories and network unavailable — \
                             run with network access to populate the cache"
                        );
                        return Ok(ExitCode::from(2));
                    }
                    eprintln!("Warning: could not fetch advisories: {e}");
                    eprintln!("Using cached advisories (may be stale)");
                }
            }
        }
    }

    run_matcher_and_output(
        &db,
        &all_packages,
        &all_warnings,
        &all_risky,
        &path,
        format,
        &cfg,
    )
}

fn run_matcher_and_output(
    db: &AdvisoryDb,
    packages: &[ripley_core::lockfile::InstalledPackage],
    warnings: &[ripley_core::lockfile::LockfileWarning],
    risky_specs: &[ripley_core::lockfile::RiskySpec],
    path: &std::path::Path,
    format: &str,
    cfg: &config::Config,
) -> anyhow::Result<ExitCode> {
    let all_advisories = db.get_all_advisories()?;
    let matches = matcher::find_matches(&all_advisories, packages, path);

    match format {
        "json" => {
            let output = serde_json::json!({
                "matches": matches.iter().map(|m| serde_json::json!({
                    "advisory_id": m.advisory.id,
                    "package": m.package.name,
                    "version": m.package.version.to_string(),
                    "severity": m.advisory.severity,
                    "summary": m.advisory.summary,
                    "project_path": m.project_path,
                })).collect::<Vec<_>>(),
                "posture_warnings": warnings.iter().map(|w| serde_json::json!({
                    "package": w.package,
                    "field": w.field,
                    "message": w.message,
                    "severity": w.severity,
                })).collect::<Vec<_>>(),
                "risky_specs": risky_specs.iter().map(|r| serde_json::json!({
                    "package": r.package,
                    "specifier": r.specifier,
                    "reason": r.reason,
                })).collect::<Vec<_>>(),
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            if matches.is_empty() {
                println!("{}", "✓ No known vulnerabilities found.".green().bold());
            } else {
                println!(
                    "{}",
                    format!(
                        "Found {} vulnerabilit{}",
                        matches.len(),
                        if matches.len() == 1 { "y" } else { "ies" }
                    )
                    .red()
                    .bold()
                );
                println!();
                println!(
                    "  {:<14} {:<30} {:<12} {:<10} SUMMARY",
                    "ADVISORY", "PACKAGE", "VERSION", "SEVERITY"
                );
                println!("  {}", "─".repeat(90));
                for m in &matches {
                    println!(
                        "  {:<14} {:<30} {:<12} {:<10} {}",
                        m.advisory.id,
                        m.package.name,
                        m.package.version.to_string(),
                        format!("{}", severity_colored(&m.advisory.severity)),
                        m.advisory.summary,
                    );
                }
            }

            if !warnings.is_empty() || !risky_specs.is_empty() {
                println!();
                let mut parts = Vec::new();
                let risky_count = risky_specs.len();
                let integrity_count = warnings
                    .iter()
                    .filter(|w| w.message.contains("integrity"))
                    .count();
                let http_count = warnings
                    .iter()
                    .filter(|w| w.message.contains("HTTP"))
                    .count();

                if risky_count > 0 {
                    parts.push(format!(
                        "{risky_count} package{} use{} non-registry sources",
                        if risky_count == 1 { "" } else { "s" },
                        if risky_count == 1 { "s" } else { "" }
                    ));
                }
                if integrity_count > 0 {
                    parts.push(format!(
                        "{integrity_count} entr{} missing or weak integrity hashes",
                        if integrity_count == 1 { "y" } else { "ies" }
                    ));
                }
                if http_count > 0 {
                    parts.push(format!(
                        "{http_count} package{} resolved over HTTP",
                        if http_count == 1 { "" } else { "s" }
                    ));
                }

                if !parts.is_empty() {
                    println!("  {} {}", "Posture:".yellow().bold(), parts.join(", "));
                }
            }
        }
    }

    let has_matches = !matches.is_empty();
    let strict_posture_fail =
        cfg.posture.strict && (!warnings.is_empty() || !risky_specs.is_empty());

    if has_matches || strict_posture_fail {
        Ok(ExitCode::from(1))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

fn cmd_status() -> anyhow::Result<()> {
    let data_dir = ripley_core::dirs::data_dir()?;
    let db_path = data_dir.join("advisories.redb");

    let cache_age = if db_path.exists() {
        let db = AdvisoryDb::open(&db_path)?;
        match db.get_last_poll()? {
            Some(ts) => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let age_secs = now.saturating_sub(ts);
                if age_secs < 60 {
                    format!("{age_secs}s ago")
                } else if age_secs < 3600 {
                    format!("{}m ago", age_secs / 60)
                } else if age_secs < 86400 {
                    format!("{}h ago", age_secs / 3600)
                } else {
                    format!("{}d ago", age_secs / 86400)
                }
            }
            None => "never".to_string(),
        }
    } else {
        "never".to_string()
    };

    let shim_path = data_dir.join("bin").join("npm");
    let shim_status = if shim_path.exists() {
        "installed"
    } else {
        "not installed"
    };

    let cwd = std::env::current_dir()?;
    let cfg = config::load_config(&cwd)?;
    let roots = if cfg.monitoring.project_roots.is_empty() {
        "none configured".to_string()
    } else {
        cfg.monitoring.project_roots.join(", ")
    };

    println!("Ripley Status");
    println!("─────────────────────────────");
    println!("  Advisory cache:   {cache_age}");
    println!("  Guard shim:       {shim_status}");
    println!("  Project roots:    {roots}");
    println!("  Daemon:           not running");

    Ok(())
}

fn cmd_config(path: bool, show: bool, init: bool) -> anyhow::Result<()> {
    let config_path = config::config_file_path()?;

    if path {
        println!("{}", config_path.display());
        return Ok(());
    }

    if init {
        if config_path.exists() {
            println!("Config already exists at {}", config_path.display());
        } else {
            config::write_default_config(&config_path)?;
            println!("Created default config at {}", config_path.display());
        }
        return Ok(());
    }

    if show {
        let cwd = std::env::current_dir()?;
        let cfg = config::load_config(&cwd)?;
        let toml_str = toml::to_string_pretty(&cfg)?;
        println!("{toml_str}");
        return Ok(());
    }

    let editor = std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| "open".to_string());

    if !config_path.exists() {
        config::write_default_config(&config_path)?;
    }

    std::process::Command::new(&editor)
        .arg(&config_path)
        .status()?;

    Ok(())
}

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Scan {
            path,
            format,
            deep,
            fix,
            no_cache,
        } => {
            let target = path.unwrap_or_else(|| PathBuf::from("."));
            match cmd_scan(target, &format, deep, fix, no_cache).await {
                Ok(code) => return code,
                Err(e) => Err(e),
            }
        }
        Commands::Guard { command } => {
            match command {
                GuardCommands::Install => println!("Guard install — not yet implemented"),
                GuardCommands::Uninstall => println!("Guard uninstall — not yet implemented"),
                GuardCommands::Status => println!("Guard status — not yet implemented"),
                GuardCommands::Trust { package } => {
                    println!("Trusting {package} — not yet implemented");
                }
                GuardCommands::Untrust { package } => {
                    println!("Untrusting {package} — not yet implemented");
                }
                GuardCommands::Log => println!("Guard log — not yet implemented"),
            }
            Ok(())
        }
        Commands::Status => cmd_status(),
        Commands::Config { path, show, init } => cmd_config(path, show, init),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e:#}");
            ExitCode::from(2)
        }
    }
}
