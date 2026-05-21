use crate::audit::TrafficLight;
use crate::lockfile;

use super::{DetectedPm, HardenFinding};

pub fn check_dependency_pinning(pm: &DetectedPm) -> Vec<HardenFinding> {
    let mut findings = Vec::new();

    findings.push(check_save_exact(pm));
    findings.push(check_lockfile_committed(pm));
    findings.push(check_integrity_hashes(pm));
    findings.push(check_exotic_sources(pm));

    findings
}

pub fn check_save_exact(pm: &DetectedPm) -> HardenFinding {
    if pm.name == "npm" || pm.name == "pnpm" {
        let output = std::process::Command::new(&pm.name)
            .args(["config", "get", "save-exact"])
            .output();

        match output {
            Ok(o) if o.status.success() => {
                let val = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if val == "true" {
                    HardenFinding {
                        name: "save-exact".to_string(),
                        status: TrafficLight::Green,
                        detail: "Exact versions enabled".to_string(),
                        fix_command: None,
                        pm: pm.name.clone(),
                    }
                } else {
                    HardenFinding {
                        name: "save-exact".to_string(),
                        status: TrafficLight::Yellow,
                        detail: "save-exact not enabled — versions may use ranges".to_string(),
                        fix_command: Some(format!("{} config set save-exact true", pm.name)),
                        pm: pm.name.clone(),
                    }
                }
            }
            _ => HardenFinding {
                name: "save-exact".to_string(),
                status: TrafficLight::Yellow,
                detail: "Could not check save-exact setting".to_string(),
                fix_command: Some(format!("{} config set save-exact true", pm.name)),
                pm: pm.name.clone(),
            },
        }
    } else {
        HardenFinding {
            name: "save-exact".to_string(),
            status: TrafficLight::Green,
            detail: format!("{} uses exact versions by default", pm.name),
            fix_command: None,
            pm: pm.name.clone(),
        }
    }
}

pub fn check_lockfile_committed(pm: &DetectedPm) -> HardenFinding {
    let output = std::process::Command::new("git")
        .args(["ls-files", &pm.lockfile_path.to_string_lossy()])
        .output();

    match output {
        Ok(o) if !String::from_utf8_lossy(&o.stdout).trim().is_empty() => HardenFinding {
            name: "Lockfile committed".to_string(),
            status: TrafficLight::Green,
            detail: format!(
                "{} is tracked by git",
                pm.lockfile_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
            ),
            fix_command: None,
            pm: pm.name.clone(),
        },
        _ => HardenFinding {
            name: "Lockfile committed".to_string(),
            status: TrafficLight::Red,
            detail: format!(
                "{} is not committed to git",
                pm.lockfile_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
            ),
            fix_command: Some(format!(
                "git add {}",
                pm.lockfile_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
            )),
            pm: pm.name.clone(),
        },
    }
}

pub fn check_integrity_hashes(pm: &DetectedPm) -> HardenFinding {
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

    let parsed = lockfile::parse_lockfile(&pm.lockfile_path);
    match parsed {
        Ok(lockfile) => {
            let total = lockfile.packages.len();
            if total == 0 {
                return HardenFinding {
                    name: "Integrity hashes".to_string(),
                    status: TrafficLight::Green,
                    detail: "No packages to check".to_string(),
                    fix_command: None,
                    pm: pm.name.clone(),
                };
            }

            let with_integrity = content.matches("integrity").count();
            let ratio = if total > 0 {
                with_integrity as f64 / total as f64
            } else {
                1.0
            };

            if ratio >= 1.0 {
                HardenFinding {
                    name: "Integrity hashes".to_string(),
                    status: TrafficLight::Green,
                    detail: format!("All {total} packages have integrity hashes"),
                    fix_command: None,
                    pm: pm.name.clone(),
                }
            } else if ratio >= 0.9 {
                HardenFinding {
                    name: "Integrity hashes".to_string(),
                    status: TrafficLight::Yellow,
                    detail: format!(
                        "{with_integrity}/{total} packages have integrity hashes ({:.0}%)",
                        ratio * 100.0
                    ),
                    fix_command: Some(format!("{} install", pm.name)),
                    pm: pm.name.clone(),
                }
            } else {
                HardenFinding {
                    name: "Integrity hashes".to_string(),
                    status: TrafficLight::Red,
                    detail: format!(
                        "{with_integrity}/{total} packages have integrity hashes ({:.0}%)",
                        ratio * 100.0
                    ),
                    fix_command: Some(format!("{} install", pm.name)),
                    pm: pm.name.clone(),
                }
            }
        }
        Err(_) => HardenFinding {
            name: "Integrity hashes".to_string(),
            status: TrafficLight::Yellow,
            detail: "Could not parse lockfile for integrity check".to_string(),
            fix_command: None,
            pm: pm.name.clone(),
        },
    }
}

pub fn check_exotic_sources(pm: &DetectedPm) -> HardenFinding {
    let parsed = lockfile::parse_lockfile(&pm.lockfile_path);
    match parsed {
        Ok(lockfile) if lockfile.risky_specs.is_empty() => HardenFinding {
            name: "Exotic sources".to_string(),
            status: TrafficLight::Green,
            detail: "No git+, http, or file: sources found".to_string(),
            fix_command: None,
            pm: pm.name.clone(),
        },
        Ok(lockfile) => {
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
                pm: pm.name.clone(),
            }
        }
        Err(_) => HardenFinding {
            name: "Exotic sources".to_string(),
            status: TrafficLight::Yellow,
            detail: "Could not parse lockfile".to_string(),
            fix_command: None,
            pm: pm.name.clone(),
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
        }
    }

    #[test]
    fn test_save_exact_yarn_default() {
        let pm = mock_pm("yarn");
        let finding = check_save_exact(&pm);
        assert_eq!(finding.status, TrafficLight::Green);
    }

    #[test]
    fn test_save_exact_bun_default() {
        let pm = mock_pm("bun");
        let finding = check_save_exact(&pm);
        assert_eq!(finding.status, TrafficLight::Green);
    }

    #[test]
    fn test_lockfile_not_committed() {
        let pm = mock_pm("npm");
        let finding = check_lockfile_committed(&pm);
        assert_eq!(finding.status, TrafficLight::Red);
    }

    #[test]
    fn test_exotic_sources_unreadable() {
        let pm = mock_pm("npm");
        let finding = check_exotic_sources(&pm);
        assert_eq!(finding.status, TrafficLight::Yellow);
    }

    #[test]
    fn test_integrity_unreadable() {
        let pm = mock_pm("npm");
        let finding = check_integrity_hashes(&pm);
        assert_eq!(finding.status, TrafficLight::Yellow);
    }
}
