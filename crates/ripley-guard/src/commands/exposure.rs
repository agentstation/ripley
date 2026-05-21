use std::process::ExitCode;

use ripley_core::forensic::exposure::{self, CveExposureReport};
use ripley_core::forensic::ioc::IocProfileSet;
use ripley_core::platform;
use ripley_core::types::Severity;

pub async fn cmd_exposure(cve_id: &str, format: &str) -> anyhow::Result<ExitCode> {
    let home = platform::home_dir()
        .ok_or_else(|| anyhow::anyhow!("could not determine home directory"))?;

    let compiled = IocProfileSet::load_compiled()?;
    let config_dir = ripley_core::dirs::config_dir()?;
    let user_profiles = IocProfileSet::load_user_profiles(&config_dir.join("iocs"))?;
    let profiles = IocProfileSet::merge(compiled, user_profiles);

    let report = exposure::assess_cve_exposure(cve_id, &profiles, &home)?;

    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&report)?;
            println!("{json}");
        }
        _ => {
            print_table(&report);
        }
    }

    let has_risk = !report.at_risk.is_empty();
    if has_risk {
        Ok(ExitCode::from(1))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

fn print_table(report: &CveExposureReport) {
    println!();
    println!("  Credential Exposure Assessment: {}", report.cve_id);
    println!("  Attack: {}", report.attack_name);
    println!("  ─────────────────────────────────────────");

    if let Some(ref warning) = report.dead_man_switch {
        println!();
        println!("  \x1b[31m⚠ DEAD MAN SWITCH DETECTED\x1b[0m");
        println!("  {warning}");
    }

    if !report.at_risk.is_empty() {
        println!();
        println!("  \x1b[31mAT RISK:\x1b[0m");
        for risk in &report.at_risk {
            let priority_color = match risk.priority {
                Severity::Critical => "\x1b[31m",
                Severity::High => "\x1b[31m",
                Severity::Medium => "\x1b[33m",
                Severity::Low => "\x1b[32m",
            };
            let reset = "\x1b[0m";
            let secret_tag = if risk.contains_secret {
                " (secret detected)"
            } else {
                ""
            };
            println!(
                "    {priority_color}[{:?}]{reset} {}{}",
                risk.priority,
                risk.path.display(),
                secret_tag,
            );
            println!("      → {}", risk.rotation_command);
        }
    }

    if !report.clean.is_empty() {
        println!();
        println!("  \x1b[32mCLEAN:\x1b[0m");
        for item in &report.clean {
            println!("    ● {item}");
        }
    }

    println!();
}
