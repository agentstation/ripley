use std::path::PathBuf;
use std::process::ExitCode;

use ripley_core::audit::TrafficLight;
use ripley_core::harden::{self, HardenReport};
use ripley_core::platform;

pub async fn cmd_harden(path: PathBuf, format: &str, fix: bool) -> anyhow::Result<ExitCode> {
    let home = platform::home_dir()
        .ok_or_else(|| anyhow::anyhow!("could not determine home directory"))?;

    let report = harden::run_harden(&path, &home);

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
        let indicator = match cat.overall {
            TrafficLight::Green => "●",
            TrafficLight::Yellow => "◐",
            TrafficLight::Red => "○",
        };
        let color = match cat.overall {
            TrafficLight::Green => "\x1b[32m",
            TrafficLight::Yellow => "\x1b[33m",
            TrafficLight::Red => "\x1b[31m",
        };
        let reset = "\x1b[0m";

        println!(
            "  {color}{indicator}{reset} {} [{color}{}{reset}]",
            cat.category, cat.overall
        );

        for finding in &cat.findings {
            let f_color = match finding.status {
                TrafficLight::Green => "\x1b[32m",
                TrafficLight::Yellow => "\x1b[33m",
                TrafficLight::Red => "\x1b[31m",
            };
            let f_indicator = match finding.status {
                TrafficLight::Green => "●",
                TrafficLight::Yellow => "◐",
                TrafficLight::Red => "○",
            };

            let pm_tag = if finding.pm.is_empty() {
                String::new()
            } else {
                format!(" [{}]", finding.pm)
            };

            println!(
                "      {f_color}{f_indicator}{reset} {}{pm_tag}: {}",
                finding.name, finding.detail
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
        green_count, yellow_count, red_count
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
