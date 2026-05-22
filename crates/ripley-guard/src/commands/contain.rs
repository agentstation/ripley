use std::process::ExitCode;

use anyhow::Result;

use ripley_core::monitor::contain;

pub async fn cmd_contain(target: &str, format: &str) -> Result<ExitCode> {
    let pid: u32 = target
        .parse()
        .map_err(|_| anyhow::anyhow!("target must be a numeric PID, got: {target}"))?;

    let data_dir = ripley_core::dirs::data_dir()?;

    eprintln!("ripley: snapshotting process {pid}...");

    let result =
        tokio::task::spawn_blocking(move || contain::contain_process(pid, &data_dir)).await??;

    if format == "json" {
        let json = serde_json::to_string_pretty(&result)?;
        println!("{json}");
    } else {
        if result.killed {
            eprintln!("ripley: process {pid} killed");
        } else {
            eprintln!("ripley: warning: failed to kill process {pid}");
        }

        eprintln!(
            "ripley: snapshot saved to {}",
            result.snapshot_path.display()
        );

        println!(
            "Process:    {} (PID {})",
            result.snapshot.name, result.snapshot.pid
        );
        println!("Command:    {}", result.snapshot.cmdline);
        println!("Open files: {}", result.snapshot.open_files.len());
        println!("Children:   {}", result.snapshot.children.len());
        println!(
            "Network:    {} connections",
            result.snapshot.network_connections.len()
        );
        println!("Killed:     {}", if result.killed { "yes" } else { "no" });
    }

    Ok(ExitCode::SUCCESS)
}
