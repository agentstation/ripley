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
            match commands::scan::cmd_scan(target, &format, deep, fix, no_cache).await {
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
        Commands::Status => commands::status::cmd_status(),
        Commands::Config { path, show, init } => commands::config::cmd_config(path, show, init),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e:#}");
            ExitCode::from(2)
        }
    }
}
