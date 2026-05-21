use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Context;
use colored::Colorize;
use ripley_core::audit::TrafficLight;
use ripley_core::harden::{self, HardenReport};
use ripley_core::platform;

use super::{traffic_light_colored, traffic_light_indicator};

pub async fn cmd_harden(path: PathBuf, format: &str, fix: bool) -> anyhow::Result<ExitCode> {
    let home = platform::home_dir()
        .ok_or_else(|| anyhow::anyhow!("could not determine home directory"))?;

    let path_clone = path.clone();
    let home_clone = home.clone();
    let report = tokio::task::spawn_blocking(move || harden::run_harden(&path_clone, &home_clone))
        .await
        .context("harden task panicked")?;

    if report.detected_pms.is_empty() {
        match format {
            "json" => {
                let json = serde_json::to_string_pretty(&report)?;
                println!("{json}");
            }
            _ => {
                println!();
                println!("  No package managers detected in {}", path.display());
                println!("  (looking for package-lock.json, pnpm-lock.yaml, yarn.lock, bun.lockb)");
                println!();
            }
        }
        return Ok(ExitCode::SUCCESS);
    }

    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&report)?;
            println!("{json}");
        }
        _ => {
            print_table(&report);
        }
    }

    if fix {
        launch_fix(&report)?;
    }

    let has_issues = report
        .categories
        .iter()
        .any(|c| c.overall != TrafficLight::Green);

    if has_issues {
        Ok(ExitCode::from(1))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

fn print_table(report: &HardenReport) {
    println!();
    println!("  Package Manager Hardening");
    println!("  ─────────────────────────────────────────");
    println!();

    println!("  Detected:");
    for pm in &report.detected_pms {
        let ver = pm.version.as_deref().unwrap_or("unknown");
        println!("    {} v{ver} ({})", pm.name, pm.lockfile_path.display());
    }
    println!();

    for cat in &report.categories {
        let indicator = traffic_light_indicator(&cat.overall);
        let colored_indicator = traffic_light_colored(indicator, &cat.overall);
        let colored_status = traffic_light_colored(&cat.overall.to_string(), &cat.overall);

        println!(
            "  {} {} [{}]",
            colored_indicator, cat.category, colored_status
        );

        for finding in &cat.findings {
            let f_indicator = traffic_light_indicator(&finding.status);
            let colored_f_indicator = traffic_light_colored(f_indicator, &finding.status);

            let pm_tag = if finding.pm.is_empty() {
                String::new()
            } else {
                format!(" [{}]", finding.pm)
            };

            println!(
                "      {} {}{pm_tag}: {}",
                colored_f_indicator, finding.name, finding.detail
            );
            if let Some(fix) = &finding.fix_command {
                println!("        → {fix}");
            }
        }
        println!();
    }

    let red_count = report
        .categories
        .iter()
        .filter(|c| c.overall == TrafficLight::Red)
        .count();
    let yellow_count = report
        .categories
        .iter()
        .filter(|c| c.overall == TrafficLight::Yellow)
        .count();
    let green_count = report
        .categories
        .iter()
        .filter(|c| c.overall == TrafficLight::Green)
        .count();

    println!(
        "  Summary: {} green, {} yellow, {} red",
        green_count.to_string().green(),
        yellow_count.to_string().yellow(),
        red_count.to_string().red()
    );
    println!();
}

fn launch_fix(report: &HardenReport) -> anyhow::Result<()> {
    let prompt = ripley_core::prompt::generate_harden_prompt(report);

    match ripley_core::harness::detect_harness() {
        Some(harness) => {
            println!("Launching {} with hardening findings...", harness.name());
            let cwd = std::env::current_dir()?;
            ripley_core::harness::launch(&harness, &prompt, &cwd)?;
        }
        None => {
            println!("No AI harness detected. Printing remediation prompt:\n");
            println!("{prompt}");
        }
    }

    Ok(())
}
