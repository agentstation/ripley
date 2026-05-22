mod commands;
mod extractor;
mod output;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

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
        /// Output format (table, json, or sarif)
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
        /// CI mode: auto-detect CI, write SARIF sidecar file
        #[arg(long)]
        ci: bool,
        /// Path to write SARIF output file (used with --ci)
        #[arg(long)]
        sarif_output: Option<PathBuf>,
    },
    /// Manage the package manager guard
    Guard {
        #[command(subcommand)]
        command: GuardCommands,
    },
    /// Show status of Ripley and monitored projects
    Status,
    /// Start background daemon (poller, watcher, IPC server)
    Watch {
        /// Fork to background and log to file
        #[arg(long)]
        daemon: bool,
    },
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
    /// Run environment security audit
    Audit {
        /// Output format (table or json)
        #[arg(long, default_value = "table")]
        format: String,
        /// Generate AI-powered remediation for findings
        #[arg(long)]
        fix: bool,
    },
    /// Check and harden package manager security settings
    Harden {
        /// Path to check (defaults to current directory)
        path: Option<PathBuf>,
        /// Output format (table or json)
        #[arg(long, default_value = "table")]
        format: String,
        /// Generate AI-powered remediation for findings
        #[arg(long)]
        fix: bool,
    },
    /// Generate AI-powered fix for a specific CVE
    Fix {
        /// CVE, GHSA, or advisory ID to remediate
        cve: String,
        /// Path to scan (defaults to current directory)
        path: Option<PathBuf>,
        /// Output format (table or json)
        #[arg(long, default_value = "table")]
        format: String,
    },
    /// Assess credential exposure for a specific CVE
    Exposure {
        /// CVE, GHSA, or advisory ID to assess
        cve: String,
        /// Output format (table or json)
        #[arg(long, default_value = "table")]
        format: String,
    },
    /// Start real-time monitoring for active compromise
    Monitor {
        /// Fork to background and log to file
        #[arg(long)]
        daemon: bool,
        /// Output format (table or json)
        #[arg(long, default_value = "table")]
        format: String,
    },
    /// Kill a suspicious process and snapshot its state
    Contain {
        /// Process ID or package name to contain
        target: String,
        /// Output format (table or json)
        #[arg(long, default_value = "table")]
        format: String,
    },
    /// Manage community detection rule sources
    Rule {
        #[command(subcommand)]
        command: RuleCommands,
    },
}

#[derive(Subcommand)]
enum RuleCommands {
    /// Add a rule source
    Add {
        /// Source name
        name: String,
        /// Source URL (base URL containing index.toml)
        url: String,
    },
    /// Remove a rule source
    Remove {
        /// Source name to remove
        name: String,
    },
    /// Fetch/update rules from a source (or all sources)
    Update {
        /// Source name (omit to update all)
        name: Option<String>,
    },
    /// List configured rule sources
    List,
    /// Trust a rule source (allows blocking in strict mode)
    Trust {
        /// Source name to trust
        name: String,
    },
    /// Untrust a rule source
    Untrust {
        /// Source name to untrust
        name: String,
    },
    /// Search rules by keyword
    Search {
        /// Search query
        query: String,
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
            ci,
            sarif_output,
        } => {
            let target = path.unwrap_or_else(|| PathBuf::from("."));
            match commands::scan::cmd_scan(target, &format, deep, fix, no_cache, ci, sarif_output)
                .await
            {
                Ok(code) => return code,
                Err(e) => Err(e),
            }
        }
        Commands::Guard { command } => match command {
            GuardCommands::Install => commands::guard::cmd_install(),
            GuardCommands::Uninstall => commands::guard::cmd_uninstall(),
            GuardCommands::Status => commands::guard::cmd_status(),
            GuardCommands::Trust { package } => commands::guard::cmd_trust(&package),
            GuardCommands::Untrust { package } => commands::guard::cmd_untrust(&package),
            GuardCommands::Log => commands::guard::cmd_log(),
        },
        Commands::Watch { daemon } => commands::watch::cmd_watch(daemon).await,
        Commands::Status => commands::status::cmd_status(),
        Commands::Config { path, show, init } => commands::config::cmd_config(path, show, init),
        Commands::Audit { format, fix } => match commands::audit::cmd_audit(&format, fix).await {
            Ok(code) => return code,
            Err(e) => Err(e),
        },
        Commands::Harden { path, format, fix } => {
            let target = path.unwrap_or_else(|| PathBuf::from("."));
            match commands::harden::cmd_harden(target, &format, fix).await {
                Ok(code) => return code,
                Err(e) => Err(e),
            }
        }
        Commands::Fix { cve, path, format } => {
            match commands::fix::cmd_fix(&cve, path, &format).await {
                Ok(code) => return code,
                Err(e) => Err(e),
            }
        }
        Commands::Exposure { cve, format } => {
            match commands::exposure::cmd_exposure(&cve, &format).await {
                Ok(code) => return code,
                Err(e) => Err(e),
            }
        }
        Commands::Monitor { daemon, format } => {
            commands::monitor::cmd_monitor(daemon, &format).await
        }
        Commands::Contain { target, format } => {
            match commands::contain::cmd_contain(&target, &format).await {
                Ok(code) => return code,
                Err(e) => Err(e),
            }
        }
        Commands::Rule { command } => commands::rule::cmd_rule(command).await,
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e:#}");
            ExitCode::from(2)
        }
    }
}
