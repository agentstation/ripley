use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;
use std::process::{Command, ExitCode};

use ripley_core::analyzer;
use ripley_core::behavioral;
use ripley_core::config::{self, GuardConfig};
use ripley_core::rules::RuleSet;
use ripley_core::sandbox::{self, SandboxProfile, SandboxResult};
use ripley_core::types::{Ecosystem, Severity};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let script = match parse_script_arg(&args) {
        Some(s) => s,
        None => {
            return delegate_to_sh(&args);
        }
    };

    let rules = {
        let config_dir = ripley_core::dirs::config_dir().unwrap_or_else(|_| PathBuf::from("."));
        let data_dir = ripley_core::dirs::data_dir().unwrap_or_else(|_| PathBuf::from("."));
        match RuleSet::load_all(&config_dir, &data_dir) {
            Ok(r) => r,
            Err(_) => match RuleSet::load_compiled() {
                Ok(r) => r,
                Err(_) => return delegate_to_sh(&args),
            },
        }
    };

    let config = load_guard_config();

    let result = analyzer::analyze(&script, &rules, Ecosystem::Npm);

    if result.risk_level <= Severity::Low {
        log_decision(&script, "allowed", &result, false);
        return execute_script(&args, &script, &config);
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
        return execute_script(&args, &script, &config);
    }

    eprint!("ripley: allow this script? [y/N] ");
    let _ = io::stderr().flush();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_ok() && input.trim().eq_ignore_ascii_case("y") {
        log_decision(&script, "allowed", &result, true);
        execute_script(&args, &script, &config)
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

fn load_guard_config() -> GuardConfig {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    config::load_config(&cwd)
        .map(|c| c.guard)
        .unwrap_or_default()
}

fn execute_script(args: &[String], script: &str, config: &GuardConfig) -> ExitCode {
    if config.sandbox {
        delegate_to_sandbox(script, config)
    } else {
        delegate_to_sh(args)
    }
}

fn delegate_to_sandbox(script: &str, config: &GuardConfig) -> ExitCode {
    let package_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let ecosystem = detect_ecosystem();
    let package = std::env::var("npm_package_name").unwrap_or_default();
    let version = std::env::var("npm_package_version").unwrap_or_default();

    let mut profile = SandboxProfile::for_ecosystem(ecosystem, &package_dir);
    profile.allow_network = config.sandbox_allow_network;

    for path in &config.sandbox_writable_paths {
        profile.writable_paths.push(PathBuf::from(path));
    }

    match sandbox::execute_sandboxed(script, &[], &profile) {
        Ok(result) => {
            if !result.sandbox_violations.is_empty() {
                log_sandbox_result(script, &result);
                for v in &result.sandbox_violations {
                    eprintln!("ripley: sandbox violation: {:?} — {}", v.kind, v.detail);
                }
            }

            if let Ok(report) = behavioral::analyze_behavior(
                &package_dir,
                ecosystem,
                &result,
                &profile,
                &package,
                &version,
            ) {
                if report.risk_score > 0.0 {
                    log_behavioral_report(&report);
                }
                for anomaly in &report.anomalies {
                    if anomaly.severity >= Severity::High {
                        eprintln!(
                            "ripley: behavioral anomaly [{:?}]: {}",
                            anomaly.kind, anomaly.description
                        );
                    }
                }
            }

            ExitCode::from(result.exit_code as u8)
        }
        Err(e) => {
            eprintln!("ripley: sandbox unavailable ({e}), falling back to /bin/sh");
            let status = Command::new("/bin/sh")
                .arg("-c")
                .arg(script)
                .current_dir(&package_dir)
                .status();
            match status {
                Ok(s) => ExitCode::from(s.code().unwrap_or(1) as u8),
                Err(io_err) => {
                    eprintln!("ripley: could not execute /bin/sh: {io_err}");
                    ExitCode::from(2)
                }
            }
        }
    }
}

fn detect_ecosystem() -> Ecosystem {
    if std::env::var("npm_package_name").is_ok() || std::env::var("npm_lifecycle_event").is_ok() {
        Ecosystem::Npm
    } else if std::env::var("CARGO_PKG_NAME").is_ok() {
        Ecosystem::Cargo
    } else if std::env::var("VIRTUAL_ENV").is_ok() || std::env::var("PIP_PREFIX").is_ok() {
        Ecosystem::PyPI
    } else {
        Ecosystem::Npm
    }
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

fn log_sandbox_result(script: &str, result: &SandboxResult) {
    let data_dir = match ripley_core::dirs::data_dir() {
        Ok(d) => d,
        Err(_) => return,
    };

    let log_path = data_dir.join("guard.jsonl");
    let package = std::env::var("npm_package_name").unwrap_or_default();
    let version = std::env::var("npm_package_version").unwrap_or_default();
    let lifecycle = std::env::var("npm_lifecycle_event").unwrap_or_default();

    let violations: Vec<serde_json::Value> = result
        .sandbox_violations
        .iter()
        .map(|v| {
            serde_json::json!({
                "kind": format!("{:?}", v.kind),
                "detail": v.detail,
            })
        })
        .collect();

    let entry = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        "package": package,
        "version": version,
        "script": if lifecycle.is_empty() {
            script.chars().take(200).collect::<String>()
        } else {
            lifecycle
        },
        "action": "sandbox_executed",
        "source": "script-shell",
        "sandbox": true,
        "exit_code": result.exit_code,
        "network_blocked": result.network_blocked,
        "duration_ms": result.duration_ms,
        "violations": violations,
    });

    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        let _ = writeln!(file, "{}", entry);
    }
}

fn log_behavioral_report(report: &behavioral::BehavioralReport) {
    let data_dir = match ripley_core::dirs::data_dir() {
        Ok(d) => d,
        Err(_) => return,
    };

    let log_path = data_dir.join("guard.jsonl");

    let anomalies: Vec<serde_json::Value> = report
        .anomalies
        .iter()
        .map(|a| {
            serde_json::json!({
                "kind": format!("{:?}", a.kind),
                "severity": a.severity.to_string(),
                "description": a.description,
                "evidence": a.evidence,
            })
        })
        .collect();

    let entry = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        "package": report.package,
        "version": report.version,
        "ecosystem": report.ecosystem.to_string(),
        "action": "behavioral_analysis",
        "source": "script-shell",
        "risk_score": report.risk_score,
        "anomalies": anomalies,
        "analysis_duration_ms": report.analysis_duration_ms,
    });

    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        let _ = writeln!(file, "{}", entry);
    }
}
