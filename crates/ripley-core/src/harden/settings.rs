use crate::audit::TrafficLight;

use super::{DetectedPm, HardenFinding};

pub fn check_pm_hardening(pm: &DetectedPm) -> Vec<HardenFinding> {
    let mut findings = Vec::new();

    findings.push(check_release_age(pm));
    findings.push(check_script_policy(pm));

    if pm.name == "pnpm" {
        findings.push(check_trust_policy(pm));
        findings.push(check_exotic_subdeps(pm));
    }

    findings
}

pub fn check_release_age(pm: &DetectedPm) -> HardenFinding {
    let config_key = match pm.name.as_str() {
        "npm" => "minReleaseAge",
        "pnpm" => "minimumReleaseAge",
        _ => {
            return HardenFinding {
                name: "Release-age gating".to_string(),
                status: TrafficLight::Yellow,
                detail: format!("{} does not have native release-age gating", pm.name),
                fix_command: None,
                pm: pm.name.clone(),
            };
        }
    };

    let output = std::process::Command::new(&pm.name)
        .args(["config", "get", config_key])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let val = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if val.is_empty() || val == "undefined" || val == "null" {
                HardenFinding {
                    name: "Release-age gating".to_string(),
                    status: TrafficLight::Red,
                    detail: format!("{config_key} not set — new packages install immediately"),
                    fix_command: Some(format!("{} config set {config_key} 86400", pm.name)),
                    pm: pm.name.clone(),
                }
            } else {
                HardenFinding {
                    name: "Release-age gating".to_string(),
                    status: TrafficLight::Green,
                    detail: format!("{config_key} = {val}"),
                    fix_command: None,
                    pm: pm.name.clone(),
                }
            }
        }
        _ => HardenFinding {
            name: "Release-age gating".to_string(),
            status: TrafficLight::Red,
            detail: format!("{config_key} not configured"),
            fix_command: Some(format!("{} config set {config_key} 86400", pm.name)),
            pm: pm.name.clone(),
        },
    }
}

pub fn check_script_policy(pm: &DetectedPm) -> HardenFinding {
    match pm.name.as_str() {
        "npm" | "pnpm" => {
            let output = std::process::Command::new(&pm.name)
                .args(["config", "get", "ignore-scripts"])
                .output();

            match output {
                Ok(o) if o.status.success() => {
                    let val = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    if val == "true" {
                        HardenFinding {
                            name: "Script execution".to_string(),
                            status: TrafficLight::Green,
                            detail: "ignore-scripts is enabled".to_string(),
                            fix_command: None,
                            pm: pm.name.clone(),
                        }
                    } else {
                        HardenFinding {
                            name: "Script execution".to_string(),
                            status: TrafficLight::Yellow,
                            detail: "ignore-scripts not set — lifecycle scripts run on install"
                                .to_string(),
                            fix_command: Some(format!(
                                "{} config set ignore-scripts true",
                                pm.name
                            )),
                            pm: pm.name.clone(),
                        }
                    }
                }
                _ => HardenFinding {
                    name: "Script execution".to_string(),
                    status: TrafficLight::Yellow,
                    detail: "Could not check ignore-scripts setting".to_string(),
                    fix_command: Some(format!("{} config set ignore-scripts true", pm.name)),
                    pm: pm.name.clone(),
                },
            }
        }
        _ => HardenFinding {
            name: "Script execution".to_string(),
            status: TrafficLight::Yellow,
            detail: format!("{} script policy check not implemented", pm.name),
            fix_command: None,
            pm: pm.name.clone(),
        },
    }
}

pub fn check_trust_policy(pm: &DetectedPm) -> HardenFinding {
    let npmrc_content = pm
        .config_path
        .as_ref()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .unwrap_or_default();

    if npmrc_content.contains("trustPolicy") && npmrc_content.contains("no-downgrade") {
        HardenFinding {
            name: "Trust policy".to_string(),
            status: TrafficLight::Green,
            detail: "trustPolicy: no-downgrade is set".to_string(),
            fix_command: None,
            pm: pm.name.clone(),
        }
    } else {
        HardenFinding {
            name: "Trust policy".to_string(),
            status: TrafficLight::Yellow,
            detail: "trustPolicy not set to no-downgrade".to_string(),
            fix_command: Some("Add trustPolicy=no-downgrade to .npmrc".to_string()),
            pm: pm.name.clone(),
        }
    }
}

pub fn check_exotic_subdeps(pm: &DetectedPm) -> HardenFinding {
    let npmrc_content = pm
        .config_path
        .as_ref()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .unwrap_or_default();

    if npmrc_content.contains("blockExoticSubdeps") && npmrc_content.contains("true") {
        HardenFinding {
            name: "Block exotic subdeps".to_string(),
            status: TrafficLight::Green,
            detail: "blockExoticSubdeps is enabled".to_string(),
            fix_command: None,
            pm: pm.name.clone(),
        }
    } else {
        HardenFinding {
            name: "Block exotic subdeps".to_string(),
            status: TrafficLight::Yellow,
            detail: "blockExoticSubdeps not enabled".to_string(),
            fix_command: Some("Add blockExoticSubdeps=true to .npmrc".to_string()),
            pm: pm.name.clone(),
        }
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
            lockfile_path: PathBuf::from("fake-lockfile"),
            config_path: None,
        }
    }

    #[test]
    fn test_release_age_yarn_unsupported() {
        let pm = mock_pm("yarn");
        let finding = check_release_age(&pm);
        assert_eq!(finding.status, TrafficLight::Yellow);
    }

    #[test]
    fn test_script_policy_yarn() {
        let pm = mock_pm("yarn");
        let finding = check_script_policy(&pm);
        assert_eq!(finding.status, TrafficLight::Yellow);
    }

    #[test]
    fn test_trust_policy_no_config() {
        let pm = mock_pm("pnpm");
        let finding = check_trust_policy(&pm);
        assert_eq!(finding.status, TrafficLight::Yellow);
    }

    #[test]
    fn test_trust_policy_with_config() {
        let dir = tempfile::tempdir().expect("tempdir");
        let npmrc = dir.path().join(".npmrc");
        std::fs::write(&npmrc, "trustPolicy=no-downgrade\n").expect("write");

        let pm = DetectedPm {
            name: "pnpm".to_string(),
            version: Some("10.0.0".to_string()),
            lockfile_path: PathBuf::from("pnpm-lock.yaml"),
            config_path: Some(npmrc),
        };
        let finding = check_trust_policy(&pm);
        assert_eq!(finding.status, TrafficLight::Green);
    }

    #[test]
    fn test_exotic_subdeps_not_set() {
        let pm = mock_pm("pnpm");
        let finding = check_exotic_subdeps(&pm);
        assert_eq!(finding.status, TrafficLight::Yellow);
    }
}
