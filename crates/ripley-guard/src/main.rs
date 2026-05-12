use clap::{Parser, Subcommand};
use std::path::PathBuf;

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
        /// Run deep forensic scan
        #[arg(long)]
        deep: bool,
    },
    /// Manage the package manager guard
    Guard {
        #[command(subcommand)]
        command: GuardCommands,
    },
    /// Show status of monitored projects
    Status,
    /// Open configuration
    Config,
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
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Scan { path, deep } => {
            let target = path.unwrap_or_else(|| PathBuf::from("."));
            if deep {
                println!(
                    "Deep forensic scan of {} — not yet implemented",
                    target.display()
                );
            } else {
                println!("Scanning {} — not yet implemented", target.display());
            }
        }
        Commands::Guard { command } => match command {
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
        },
        Commands::Status => println!("Status — not yet implemented"),
        Commands::Config => println!("Config — not yet implemented"),
    }

    Ok(())
}
