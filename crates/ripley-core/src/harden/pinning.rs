use std::path::Path;

use crate::audit::TrafficLight;
use crate::lockfile;

use super::{DetectedPm, HardenFinding};

/// JS-ecosystem package managers that support pinning checks.
const JS_ECOSYSTEMS: &[&str] = &["npm", "pnpm", "yarn", "bun"];

fn is_js_ecosystem(pm_name: &str) -> bool {
    JS_ECOSYSTEMS.contains(&pm_name)
}

pub fn check_dependency_pinning(pm: &DetectedPm) -> Vec<HardenFinding> {
    vec![
        check_save_exact(pm),
        check_lockfile_committed(pm),
        check_integrity_hashes(pm),
        check_exotic_sources(pm),
    ]
}

// --- save-exact ---

pub fn check_save_exact(pm: &DetectedPm) -> HardenFinding {
    if pm.name == "npm" || pm.name == "pnpm" {
        let config_output = std::process::Command::new(&pm.name)
            .args(["config", "get", "save-exact"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
        evaluate_save_exact(&pm.name, config_output.as_deref())
    } else {
        evaluate_save_exact(&pm.name, None)
    }
}

pub fn evaluate_save_exact(pm_name: &str, config_value: Option<&str>) -> HardenFinding {
    match pm_name {
        "npm" | "pnpm" => match config_value {
            Some("true") => HardenFinding {
                name: "save-exact".to_string(),
                status: TrafficLight::Green,
                detail: "Exact versions enabled".to_string(),
                fix_command: None,
                pm: pm_name.to_string(),
            },
            Some(_) => HardenFinding {
                name: "save-exact".to_string(),
                status: TrafficLight::Yellow,
                detail: "save-exact not enabled — versions may use ranges".to_string(),
                fix_command: Some(format!("{pm_name} config set save-exact true")),
                pm: pm_name.to_string(),
            },
            None => HardenFinding {
                name: "save-exact".to_string(),
                status: TrafficLight::Yellow,
                detail: "Could not check save-exact setting".to_string(),
                fix_command: Some(format!("{pm_name} config set save-exact true")),
                pm: pm_name.to_string(),
            },
        },
        "yarn" => HardenFinding {
            name: "save-exact".to_string(),
            status: TrafficLight::Yellow,
            detail: "Consider configuring exact version pinning".to_string(),
            fix_command: Some("yarn config set defaultSemverRangePrefix ''".to_string()),
            pm: pm_name.to_string(),
        },
        "bun" => HardenFinding {
            name: "save-exact".to_string(),
            status: TrafficLight::Yellow,
            detail: "Consider configuring exact version pinning".to_string(),
            fix_command: Some("echo 'install.exact = true' >> bunfig.toml".to_string()),
            pm: pm_name.to_string(),
        },
        _ => HardenFinding {
            name: "save-exact".to_string(),
            status: TrafficLight::Green,
            detail: format!("Not applicable for {pm_name}"),
            fix_command: None,
            pm: pm_name.to_string(),
        },
    }
}

// --- lockfile committed ---

pub fn check_lockfile_committed(pm: &DetectedPm) -> HardenFinding {
    let project_dir = pm.lockfile_path.parent().unwrap_or(Path::new("."));
    let lockfile_name = pm
        .lockfile_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();

    if lockfile_name.is_empty() {
        return evaluate_lockfile_committed(&pm.name, lockfile_name, None);
    }

    let output = std::process::Command::new("git")
        .current_dir(project_dir)
        .args(["ls-files", lockfile_name])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

    evaluate_lockfile_committed(&pm.name, lockfile_name, output.as_deref())
}

pub fn evaluate_lockfile_committed(
    pm_name: &str,
    lockfile_name: &str,
    git_output: Option<&str>,
) -> HardenFinding {
    match git_output {
        Some(out) if !out.is_empty() => HardenFinding {
            name: "Lockfile committed".to_string(),
            status: TrafficLight::Green,
            detail: format!("{lockfile_name} is tracked by git"),
            fix_command: None,
            pm: pm_name.to_string(),
        },
        _ => HardenFinding {
            name: "Lockfile committed".to_string(),
            status: TrafficLight::Red,
            detail: format!("{lockfile_name} is not committed to git"),
            fix_command: Some(format!("git add {lockfile_name}")),
            pm: pm_name.to_string(),
        },
    }
}

// --- integrity hashes ---

pub fn check_integrity_hashes(pm: &DetectedPm) -> HardenFinding {
    // Binary lockfiles (bun.lockb) cannot be inspected as text
    if pm.binary {
        return HardenFinding {
            name: "Integrity hashes".to_string(),
            status: TrafficLight::Yellow,
            detail: "Binary lockfile — cannot inspect".to_string(),
            fix_command: None,
            pm: pm.name.clone(),
        };
    }

    // Non-JS ecosystems: basic check not applicable
    if !is_js_ecosystem(&pm.name) {
        return HardenFinding {
            name: "Integrity hashes".to_string(),
            status: TrafficLight::Green,
            detail: format!("Not applicable for {}", pm.name),
            fix_command: None,
            pm: pm.name.clone(),
        };
    }

    // Read content once
    let content = match std::fs::read_to_string(&pm.lockfile_path) {
        Ok(c) => c,
        Err(_) => {
            return HardenFinding {
                name: "Integrity hashes".to_string(),
                status: TrafficLight::Yellow,
                detail: "Could not read lockfile".to_string(),
                fix_command: None,
                pm: pm.name.clone(),
            };
        }
    };

    // Get package count from parsed lockfile
    let package_count = lockfile::parse_lockfile(&pm.lockfile_path)
        .ok()
        .map(|lf| lf.packages.len());

    evaluate_integrity_hashes(&pm.name, Some(&content), package_count)
}

pub fn evaluate_integrity_hashes(
    pm_name: &str,
    content: Option<&str>,
    package_count: Option<usize>,
) -> HardenFinding {
    let content = match content {
        Some(c) => c,
        None => {
            return HardenFinding {
                name: "Integrity hashes".to_string(),
                status: TrafficLight::Yellow,
                detail: "Could not read lockfile".to_string(),
                fix_command: None,
                pm: pm_name.to_string(),
            };
        }
    };

    let total = match package_count {
        Some(n) => n,
        None => {
            return HardenFinding {
                name: "Integrity hashes".to_string(),
                status: TrafficLight::Yellow,
                detail: "Could not parse lockfile for integrity check".to_string(),
                fix_command: None,
                pm: pm_name.to_string(),
            };
        }
    };

    if total == 0 {
        return HardenFinding {
            name: "Integrity hashes".to_string(),
            status: TrafficLight::Green,
            detail: "No packages to check".to_string(),
            fix_command: None,
            pm: pm_name.to_string(),
        };
    }

    let integrity_count = content
        .lines()
        .filter(|line| {
            let t = line.trim();
            t.contains("integrity")
                && (t.contains("sha512-") || t.contains("sha256-") || t.contains("sha1-"))
        })
        .count();

    let ratio = integrity_count as f64 / total as f64;

    if ratio >= 1.0 {
        HardenFinding {
            name: "Integrity hashes".to_string(),
            status: TrafficLight::Green,
            detail: format!("All {total} packages have integrity hashes"),
            fix_command: None,
            pm: pm_name.to_string(),
        }
    } else if ratio >= 0.9 {
        HardenFinding {
            name: "Integrity hashes".to_string(),
            status: TrafficLight::Yellow,
            detail: format!(
                "{integrity_count}/{total} packages have integrity hashes ({:.0}%)",
                ratio * 100.0
            ),
            fix_command: Some(format!("{pm_name} install")),
            pm: pm_name.to_string(),
        }
    } else {
        HardenFinding {
            name: "Integrity hashes".to_string(),
            status: TrafficLight::Red,
            detail: format!(
                "{integrity_count}/{total} packages have integrity hashes ({:.0}%)",
                ratio * 100.0
            ),
            fix_command: Some(format!("{pm_name} install")),
            pm: pm_name.to_string(),
        }
    }
}

// --- exotic sources ---

pub fn check_exotic_sources(pm: &DetectedPm) -> HardenFinding {
    // Binary lockfiles cannot be inspected
    if pm.binary {
        return HardenFinding {
            name: "Exotic sources".to_string(),
            status: TrafficLight::Yellow,
            detail: "Binary lockfile — cannot inspect".to_string(),
            fix_command: None,
            pm: pm.name.clone(),
        };
    }

    let parsed = lockfile::parse_lockfile(&pm.lockfile_path);
    evaluate_exotic_sources(&pm.name, parsed.ok().as_ref())
}

pub fn evaluate_exotic_sources(
    pm_name: &str,
    parsed: Option<&lockfile::ParsedLockfile>,
) -> HardenFinding {
    match parsed {
        Some(lockfile) if lockfile.risky_specs.is_empty() => HardenFinding {
            name: "Exotic sources".to_string(),
            status: TrafficLight::Green,
            detail: "No git+, http, or file: sources found".to_string(),
            fix_command: None,
            pm: pm_name.to_string(),
        },
        Some(lockfile) => {
            let count = lockfile.risky_specs.len();
            let examples: Vec<String> = lockfile
                .risky_specs
                .iter()
                .take(3)
                .map(|r| format!("{}: {}", r.package, r.reason))
                .collect();
            HardenFinding {
                name: "Exotic sources".to_string(),
                status: TrafficLight::Red,
                detail: format!("{count} exotic source(s): {}", examples.join("; ")),
                fix_command: None,
                pm: pm_name.to_string(),
            }
        }
        None => HardenFinding {
            name: "Exotic sources".to_string(),
            status: TrafficLight::Yellow,
            detail: "Could not parse lockfile".to_string(),
            fix_command: None,
            pm: pm_name.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn mock_pm(name: &str) -> DetectedPm {
        DetectedPm {
            name: name.to_string(),
            version: Some("1.0.0".to_string()),
            lockfile_path: PathBuf::from("nonexistent-lockfile"),
            config_path: None,
            binary: false,
        }
    }

    fn mock_binary_pm(name: &str) -> DetectedPm {
        DetectedPm {
            name: name.to_string(),
            version: Some("1.0.0".to_string()),
            lockfile_path: PathBuf::from("bun.lockb"),
            config_path: None,
            binary: true,
        }
    }

    // --- save-exact evaluate tests ---

    #[test]
    fn test_evaluate_save_exact_npm_true() {
        let f = evaluate_save_exact("npm", Some("true"));
        assert_eq!(f.status, TrafficLight::Green);
        assert_eq!(f.pm, "npm");
    }

    #[test]
    fn test_evaluate_save_exact_npm_false() {
        let f = evaluate_save_exact("npm", Some("false"));
        assert_eq!(f.status, TrafficLight::Yellow);
        assert!(f.fix_command.is_some());
    }

    #[test]
    fn test_evaluate_save_exact_npm_none() {
        let f = evaluate_save_exact("npm", None);
        assert_eq!(f.status, TrafficLight::Yellow);
    }

    #[test]
    fn test_evaluate_save_exact_pnpm_true() {
        let f = evaluate_save_exact("pnpm", Some("true"));
        assert_eq!(f.status, TrafficLight::Green);
    }

    #[test]
    fn test_evaluate_save_exact_yarn() {
        let f = evaluate_save_exact("yarn", None);
        assert_eq!(f.status, TrafficLight::Yellow);
        assert!(f.detail.contains("exact version pinning"));
        assert!(f.fix_command.as_deref().unwrap_or("").contains("yarn"));
    }

    #[test]
    fn test_evaluate_save_exact_bun() {
        let f = evaluate_save_exact("bun", None);
        assert_eq!(f.status, TrafficLight::Yellow);
        assert!(f.detail.contains("exact version pinning"));
        assert!(f.fix_command.as_deref().unwrap_or("").contains("bunfig"));
    }

    #[test]
    fn test_evaluate_save_exact_cargo_not_applicable() {
        let f = evaluate_save_exact("cargo", None);
        assert_eq!(f.status, TrafficLight::Green);
        assert!(f.detail.contains("Not applicable"));
    }

    #[test]
    fn test_evaluate_save_exact_go_not_applicable() {
        let f = evaluate_save_exact("go", None);
        assert_eq!(f.status, TrafficLight::Green);
        assert!(f.detail.contains("Not applicable"));
    }

    #[test]
    fn test_save_exact_yarn_default() {
        let pm = mock_pm("yarn");
        let finding = check_save_exact(&pm);
        assert_eq!(finding.status, TrafficLight::Yellow);
    }

    #[test]
    fn test_save_exact_bun_default() {
        let pm = mock_pm("bun");
        let finding = check_save_exact(&pm);
        assert_eq!(finding.status, TrafficLight::Yellow);
    }

    // --- lockfile committed evaluate tests ---

    #[test]
    fn test_evaluate_lockfile_committed_tracked() {
        let f = evaluate_lockfile_committed("npm", "package-lock.json", Some("package-lock.json"));
        assert_eq!(f.status, TrafficLight::Green);
    }

    #[test]
    fn test_evaluate_lockfile_committed_not_tracked() {
        let f = evaluate_lockfile_committed("npm", "package-lock.json", Some(""));
        assert_eq!(f.status, TrafficLight::Red);
        assert_eq!(f.fix_command.as_deref(), Some("git add package-lock.json"));
    }

    #[test]
    fn test_evaluate_lockfile_committed_no_output() {
        let f = evaluate_lockfile_committed("npm", "package-lock.json", None);
        assert_eq!(f.status, TrafficLight::Red);
    }

    #[test]
    fn test_lockfile_not_committed() {
        let pm = mock_pm("npm");
        let finding = check_lockfile_committed(&pm);
        assert_eq!(finding.status, TrafficLight::Red);
    }

    // --- integrity hashes evaluate tests ---

    #[test]
    fn test_evaluate_integrity_all_present() {
        let content = r#"
            "integrity": "sha512-abc123"
            "integrity": "sha512-def456"
        "#;
        let f = evaluate_integrity_hashes("npm", Some(content), Some(2));
        assert_eq!(f.status, TrafficLight::Green);
        assert!(f.detail.contains("All 2 packages"));
    }

    #[test]
    fn test_evaluate_integrity_none_present() {
        let content = "no hashes here\njust some text\n";
        let f = evaluate_integrity_hashes("npm", Some(content), Some(10));
        assert_eq!(f.status, TrafficLight::Red);
        assert!(f.detail.contains("0/10"));
    }

    #[test]
    fn test_evaluate_integrity_partial() {
        // 9 out of 10 -> 90% -> Yellow
        let mut lines = String::new();
        for _ in 0..9 {
            lines.push_str("  \"integrity\": \"sha512-abc\"\n");
        }
        lines.push_str("  no hash line\n");
        let f = evaluate_integrity_hashes("npm", Some(&lines), Some(10));
        assert_eq!(f.status, TrafficLight::Yellow);
    }

    #[test]
    fn test_evaluate_integrity_no_content() {
        let f = evaluate_integrity_hashes("npm", None, Some(5));
        assert_eq!(f.status, TrafficLight::Yellow);
    }

    #[test]
    fn test_evaluate_integrity_no_packages() {
        let f = evaluate_integrity_hashes("npm", Some(""), None);
        assert_eq!(f.status, TrafficLight::Yellow);
    }

    #[test]
    fn test_evaluate_integrity_zero_packages() {
        let f = evaluate_integrity_hashes("npm", Some(""), Some(0));
        assert_eq!(f.status, TrafficLight::Green);
        assert!(f.detail.contains("No packages"));
    }

    #[test]
    fn test_evaluate_integrity_naive_match_rejected() {
        // A line with "integrity" but no hash should not count
        let content = "  \"integrity\": \"unknown-format\"\n";
        let f = evaluate_integrity_hashes("npm", Some(content), Some(1));
        assert_eq!(f.status, TrafficLight::Red);
        assert!(f.detail.contains("0/1"));
    }

    #[test]
    fn test_integrity_binary_lockfile() {
        let pm = mock_binary_pm("bun");
        let finding = check_integrity_hashes(&pm);
        assert_eq!(finding.status, TrafficLight::Yellow);
        assert!(finding.detail.contains("Binary lockfile"));
    }

    #[test]
    fn test_integrity_non_js_ecosystem() {
        let pm = mock_pm("cargo");
        let finding = check_integrity_hashes(&pm);
        assert_eq!(finding.status, TrafficLight::Green);
        assert!(finding.detail.contains("Not applicable"));
    }

    #[test]
    fn test_integrity_unreadable() {
        let pm = mock_pm("npm");
        let finding = check_integrity_hashes(&pm);
        assert_eq!(finding.status, TrafficLight::Yellow);
    }

    // --- exotic sources evaluate tests ---

    #[test]
    fn test_evaluate_exotic_sources_clean() {
        let parsed = lockfile::ParsedLockfile {
            packages: vec![],
            risky_specs: vec![],
            warnings: vec![],
        };
        let f = evaluate_exotic_sources("npm", Some(&parsed));
        assert_eq!(f.status, TrafficLight::Green);
    }

    #[test]
    fn test_evaluate_exotic_sources_risky() {
        let parsed = lockfile::ParsedLockfile {
            packages: vec![],
            risky_specs: vec![lockfile::RiskySpec {
                package: "evil-pkg".to_string(),
                specifier: "git+https://evil.com".to_string(),
                reason: "git source".to_string(),
            }],
            warnings: vec![],
        };
        let f = evaluate_exotic_sources("npm", Some(&parsed));
        assert_eq!(f.status, TrafficLight::Red);
        assert!(f.detail.contains("evil-pkg"));
    }

    #[test]
    fn test_evaluate_exotic_sources_no_parse() {
        let f = evaluate_exotic_sources("npm", None);
        assert_eq!(f.status, TrafficLight::Yellow);
    }

    #[test]
    fn test_exotic_sources_binary_lockfile() {
        let pm = mock_binary_pm("bun");
        let finding = check_exotic_sources(&pm);
        assert_eq!(finding.status, TrafficLight::Yellow);
        assert!(finding.detail.contains("Binary lockfile"));
    }

    #[test]
    fn test_exotic_sources_unreadable() {
        let pm = mock_pm("npm");
        let finding = check_exotic_sources(&pm);
        assert_eq!(finding.status, TrafficLight::Yellow);
    }
}
