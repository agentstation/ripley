use colored::Colorize;

use ripley_core::lockfile::{LockfileWarning, RiskySpec};
use ripley_core::matcher::Match;
use ripley_core::types::Severity;

pub fn severity_colored(severity: &Option<Severity>) -> colored::ColoredString {
    match severity {
        Some(Severity::Critical) => "CRITICAL".red().bold(),
        Some(Severity::High) => "HIGH".red(),
        Some(Severity::Medium) => "MEDIUM".yellow(),
        Some(Severity::Low) => "LOW".blue(),
        None => "UNKNOWN".dimmed(),
    }
}

pub fn print_table(matches: &[Match], warnings: &[LockfileWarning], risky_specs: &[RiskySpec]) {
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
        for m in matches {
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

    print_posture_summary(warnings, risky_specs);
}

pub fn print_json(
    matches: &[Match],
    warnings: &[LockfileWarning],
    risky_specs: &[RiskySpec],
) -> anyhow::Result<()> {
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
    Ok(())
}

fn print_posture_summary(warnings: &[LockfileWarning], risky_specs: &[RiskySpec]) {
    if warnings.is_empty() && risky_specs.is_empty() {
        return;
    }

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
