use std::io::{self, IsTerminal, Write};
use std::process::{Command, ExitCode};

use ripley_core::analyzer;
use ripley_core::rules::RuleSet;
use ripley_core::types::{Ecosystem, Severity};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let script = match parse_script_arg(&args) {
        Some(s) => s,
        None => {
            return delegate_to_sh(&args);
        }
    };

    let rules = match RuleSet::load_compiled() {
        Ok(r) => {
            let config_dir = ripley_core::dirs::config_dir().ok();
            if let Some(dir) = config_dir {
                let user_rules_dir = dir.join("rules");
                match RuleSet::load_user_rules(&user_rules_dir) {
                    Ok(user) => RuleSet::merge(r, user),
                    Err(_) => r,
                }
            } else {
                r
            }
        }
        Err(_) => {
            return delegate_to_sh(&args);
        }
    };

    let result = analyzer::analyze(&script, &rules, Ecosystem::Npm);

    if result.risk_level <= Severity::Low {
        log_decision(&script, "allowed", &result, false);
        return delegate_to_sh(&args);
    }

    print_analysis(&result);

    if std::env::var("RIPLEY_NON_INTERACTIVE").is_ok() || !io::stdin().is_terminal() {
        if result.risk_level >= Severity::High {
            eprintln!(
                "ripley: blocked (non-interactive, risk: {})",
                result.risk_level
            );
            log_decision(&script, "blocked", &result, false);
            return ExitCode::from(1);
        }
        log_decision(&script, "allowed", &result, false);
        return delegate_to_sh(&args);
    }

    eprint!("ripley: allow this script? [y/N] ");
    let _ = io::stderr().flush();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_ok() && input.trim().eq_ignore_ascii_case("y") {
        log_decision(&script, "allowed", &result, true);
        delegate_to_sh(&args)
    } else {
        eprintln!("ripley: blocked by user");
        log_decision(&script, "blocked", &result, true);
        ExitCode::from(1)
    }
}

fn parse_script_arg(args: &[String]) -> Option<String> {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == "-c" {
            return iter.next().cloned();
        }
    }
    None
}

fn delegate_to_sh(args: &[String]) -> ExitCode {
    let status = Command::new("/bin/sh").args(args).status();
    match status {
        Ok(s) => ExitCode::from(s.code().unwrap_or(1) as u8),
        Err(e) => {
            eprintln!("ripley: could not execute /bin/sh: {e}");
            ExitCode::from(2)
        }
    }
}

fn print_analysis(result: &analyzer::AnalysisResult) {
    eprintln!();
    eprintln!("ripley: script flagged (risk: {})", result.risk_level);
    eprintln!();

    for line in &result.highlighted_lines {
        eprintln!(
            "  {:>4} │ {} ← {}",
            line.line_number, line.text, line.severity
        );
    }
    eprintln!();
}

fn log_decision(script: &str, action: &str, result: &analyzer::AnalysisResult, prompted: bool) {
    let data_dir = match ripley_core::dirs::data_dir() {
        Ok(d) => d,
        Err(_) => return,
    };

    let log_path = data_dir.join("guard.jsonl");
    let matched_rules: Vec<&str> = result
        .matched_rules
        .iter()
        .map(|r| r.rule_id.as_str())
        .collect();

    let package = std::env::var("npm_package_name").unwrap_or_default();
    let version = std::env::var("npm_package_version").unwrap_or_default();
    let lifecycle = std::env::var("npm_lifecycle_event").unwrap_or_default();

    let entry = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        "package": package,
        "version": version,
        "script": if lifecycle.is_empty() {
            script.chars().take(200).collect::<String>()
        } else {
            lifecycle
        },
        "action": action,
        "risk_level": result.risk_level,
        "matched_rules": matched_rules,
        "source": "script-shell",
        "user_decision": prompted,
    });

    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        let _ = writeln!(file, "{}", entry);
    }
}
