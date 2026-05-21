use crate::audit::TrafficLight;

use super::{DetectedPm, HardenFinding};

/// JS-ecosystem package managers that support PM hardening checks.
const JS_ECOSYSTEMS: &[&str] = &["npm", "pnpm", "yarn", "bun"];

fn is_js_ecosystem(pm_name: &str) -> bool {
    JS_ECOSYSTEMS.contains(&pm_name)
}

pub fn check_pm_hardening(pm: &DetectedPm) -> Vec<HardenFinding> {
    // Non-JS ecosystems: return basic "not applicable" findings
    if !is_js_ecosystem(&pm.name) {
        return vec![
            HardenFinding {
                name: "Release-age gating".to_string(),
                status: TrafficLight::Green,
                detail: format!("Not applicable for {}", pm.name),
                fix_command: None,
                pm: pm.name.clone(),
            },
            HardenFinding {
                name: "Script execution".to_string(),
                status: TrafficLight::Green,
                detail: format!("Not applicable for {}", pm.name),
                fix_command: None,
                pm: pm.name.clone(),
            },
        ];
    }

    let mut findings = Vec::new();

    findings.push(check_release_age(pm));
    findings.push(check_script_policy(pm));

    if pm.name == "pnpm" {
        findings.push(check_trust_policy(pm));
        findings.push(check_exotic_subdeps(pm));
    }

    findings
}

// --- release-age gating ---

pub fn check_release_age(pm: &DetectedPm) -> HardenFinding {
    let config_key = match pm.name.as_str() {
        "npm" => "minReleaseAge",
        "pnpm" => "minimumReleaseAge",
        _ => {
            return evaluate_release_age(&pm.name, None, None);
        }
    };

    let config_output = std::process::Command::new(&pm.name)
        .args(["config", "get", config_key])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

    evaluate_release_age(&pm.name, Some(config_key), config_output.as_deref())
}

pub fn evaluate_release_age(
    pm_name: &str,
    config_key: Option<&str>,
    config_value: Option<&str>,
) -> HardenFinding {
    let config_key = match config_key {
        Some(k) => k,
        None => {
            return HardenFinding {
                name: "Release-age gating".to_string(),
                status: TrafficLight::Yellow,
                detail: format!("{pm_name} does not have native release-age gating"),
                fix_command: None,
                pm: pm_name.to_string(),
            };
        }
    };

    match config_value {
        Some(val) if !val.is_empty() && val != "undefined" && val != "null" => HardenFinding {
            name: "Release-age gating".to_string(),
            status: TrafficLight::Green,
            detail: format!("{config_key} = {val}"),
            fix_command: None,
            pm: pm_name.to_string(),
        },
        _ => HardenFinding {
            name: "Release-age gating".to_string(),
            status: TrafficLight::Red,
            detail: format!("{config_key} not set — new packages install immediately"),
            fix_command: Some(format!("{pm_name} config set {config_key} 86400")),
            pm: pm_name.to_string(),
        },
    }
}

// --- script execution policy ---

pub fn check_script_policy(pm: &DetectedPm) -> HardenFinding {
    match pm.name.as_str() {
        "npm" | "pnpm" => {
            let config_output = std::process::Command::new(&pm.name)
                .args(["config", "get", "ignore-scripts"])
                .output()
                .ok()
                .filter(|o| o.status.success())
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
            evaluate_script_policy(&pm.name, config_output.as_deref())
        }
        _ => evaluate_script_policy(&pm.name, None),
    }
}

pub fn evaluate_script_policy(pm_name: &str, config_value: Option<&str>) -> HardenFinding {
    match pm_name {
        "npm" | "pnpm" => match config_value {
            Some("true") => HardenFinding {
                name: "Script execution".to_string(),
                status: TrafficLight::Green,
                detail: "ignore-scripts is enabled".to_string(),
                fix_command: None,
                pm: pm_name.to_string(),
            },
            Some(_) => HardenFinding {
                name: "Script execution".to_string(),
                status: TrafficLight::Yellow,
                detail: "ignore-scripts not set — lifecycle scripts run on install".to_string(),
                fix_command: Some(format!("{pm_name} config set ignore-scripts true")),
                pm: pm_name.to_string(),
            },
            None => HardenFinding {
                name: "Script execution".to_string(),
                status: TrafficLight::Yellow,
                detail: "Could not check ignore-scripts setting".to_string(),
                fix_command: Some(format!("{pm_name} config set ignore-scripts true")),
                pm: pm_name.to_string(),
            },
        },
        _ => HardenFinding {
            name: "Script execution".to_string(),
            status: TrafficLight::Yellow,
            detail: format!("{pm_name} script policy check not implemented"),
            fix_command: None,
            pm: pm_name.to_string(),
        },
    }
}

// --- trust policy (pnpm) ---

pub fn check_trust_policy(pm: &DetectedPm) -> HardenFinding {
    let npmrc_content = pm
        .config_path
        .as_ref()
        .and_then(|p| std::fs::read_to_string(p).ok());
    evaluate_trust_policy(&pm.name, npmrc_content.as_deref())
}

/// Parse `.npmrc` content for a `key=value` pair, handling optional whitespace around `=`.
fn npmrc_has_value(content: &str, key: &str, value: &str) -> bool {
    content.lines().any(|line| {
        let trimmed = line.trim();
        // Strip comments
        let trimmed = trimmed.split('#').next().unwrap_or("");
        if let Some((k, v)) = trimmed.split_once('=') {
            k.trim() == key && v.trim() == value
        } else {
            false
        }
    })
}

pub fn evaluate_trust_policy(pm_name: &str, npmrc_content: Option<&str>) -> HardenFinding {
    let content = npmrc_content.unwrap_or("");
    if npmrc_has_value(content, "trustPolicy", "no-downgrade") {
        HardenFinding {
            name: "Trust policy".to_string(),
            status: TrafficLight::Green,
            detail: "trustPolicy: no-downgrade is set".to_string(),
            fix_command: None,
            pm: pm_name.to_string(),
        }
    } else {
        HardenFinding {
            name: "Trust policy".to_string(),
            status: TrafficLight::Yellow,
            detail: "trustPolicy not set to no-downgrade".to_string(),
            fix_command: Some("Add trustPolicy=no-downgrade to .npmrc".to_string()),
            pm: pm_name.to_string(),
        }
    }
}

// --- exotic subdeps blocking (pnpm) ---

pub fn check_exotic_subdeps(pm: &DetectedPm) -> HardenFinding {
    let npmrc_content = pm
        .config_path
        .as_ref()
        .and_then(|p| std::fs::read_to_string(p).ok());
    evaluate_exotic_subdeps(&pm.name, npmrc_content.as_deref())
}

pub fn evaluate_exotic_subdeps(pm_name: &str, npmrc_content: Option<&str>) -> HardenFinding {
    let content = npmrc_content.unwrap_or("");
    if npmrc_has_value(content, "blockExoticSubdeps", "true") {
        HardenFinding {
            name: "Block exotic subdeps".to_string(),
            status: TrafficLight::Green,
            detail: "blockExoticSubdeps is enabled".to_string(),
            fix_command: None,
            pm: pm_name.to_string(),
        }
    } else {
        HardenFinding {
            name: "Block exotic subdeps".to_string(),
            status: TrafficLight::Yellow,
            detail: "blockExoticSubdeps not enabled".to_string(),
            fix_command: Some("Add blockExoticSubdeps=true to .npmrc".to_string()),
            pm: pm_name.to_string(),
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
            binary: false,
        }
    }

    // --- release-age evaluate tests ---

    #[test]
    fn test_evaluate_release_age_npm_set() {
        let f = evaluate_release_age("npm", Some("minReleaseAge"), Some("86400"));
        assert_eq!(f.status, TrafficLight::Green);
        assert!(f.detail.contains("86400"));
    }

    #[test]
    fn test_evaluate_release_age_npm_unset() {
        let f = evaluate_release_age("npm", Some("minReleaseAge"), Some("undefined"));
        assert_eq!(f.status, TrafficLight::Red);
    }

    #[test]
    fn test_evaluate_release_age_npm_empty() {
        let f = evaluate_release_age("npm", Some("minReleaseAge"), Some(""));
        assert_eq!(f.status, TrafficLight::Red);
    }

    #[test]
    fn test_evaluate_release_age_npm_none() {
        let f = evaluate_release_age("npm", Some("minReleaseAge"), None);
        assert_eq!(f.status, TrafficLight::Red);
    }

    #[test]
    fn test_evaluate_release_age_unsupported() {
        let f = evaluate_release_age("yarn", None, None);
        assert_eq!(f.status, TrafficLight::Yellow);
        assert!(f.detail.contains("does not have native"));
    }

    #[test]
    fn test_release_age_yarn_unsupported() {
        let pm = mock_pm("yarn");
        let finding = check_release_age(&pm);
        assert_eq!(finding.status, TrafficLight::Yellow);
    }

    // --- script policy evaluate tests ---

    #[test]
    fn test_evaluate_script_policy_enabled() {
        let f = evaluate_script_policy("npm", Some("true"));
        assert_eq!(f.status, TrafficLight::Green);
    }

    #[test]
    fn test_evaluate_script_policy_disabled() {
        let f = evaluate_script_policy("npm", Some("false"));
        assert_eq!(f.status, TrafficLight::Yellow);
    }

    #[test]
    fn test_evaluate_script_policy_none() {
        let f = evaluate_script_policy("npm", None);
        assert_eq!(f.status, TrafficLight::Yellow);
    }

    #[test]
    fn test_evaluate_script_policy_unsupported() {
        let f = evaluate_script_policy("yarn", None);
        assert_eq!(f.status, TrafficLight::Yellow);
        assert!(f.detail.contains("not implemented"));
    }

    #[test]
    fn test_script_policy_yarn() {
        let pm = mock_pm("yarn");
        let finding = check_script_policy(&pm);
        assert_eq!(finding.status, TrafficLight::Yellow);
    }

    // --- trust policy evaluate tests ---

    #[test]
    fn test_evaluate_trust_policy_set() {
        let f = evaluate_trust_policy("pnpm", Some("trustPolicy=no-downgrade\n"));
        assert_eq!(f.status, TrafficLight::Green);
    }

    #[test]
    fn test_evaluate_trust_policy_with_spaces() {
        let f = evaluate_trust_policy("pnpm", Some("trustPolicy = no-downgrade\n"));
        assert_eq!(f.status, TrafficLight::Green);
    }

    #[test]
    fn test_evaluate_trust_policy_not_set() {
        let f = evaluate_trust_policy("pnpm", None);
        assert_eq!(f.status, TrafficLight::Yellow);
    }

    #[test]
    fn test_evaluate_trust_policy_wrong_value() {
        let f = evaluate_trust_policy("pnpm", Some("trustPolicy=permissive\n"));
        assert_eq!(f.status, TrafficLight::Yellow);
    }

    #[test]
    fn test_evaluate_trust_policy_false_positive_rejected() {
        // "trustPolicy" appears but not as a proper key=value for no-downgrade
        let f = evaluate_trust_policy(
            "pnpm",
            Some("# trustPolicy is important\nother=no-downgrade\n"),
        );
        assert_eq!(f.status, TrafficLight::Yellow);
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
            binary: false,
        };
        let finding = check_trust_policy(&pm);
        assert_eq!(finding.status, TrafficLight::Green);
    }

    // --- exotic subdeps evaluate tests ---

    #[test]
    fn test_evaluate_exotic_subdeps_enabled() {
        let f = evaluate_exotic_subdeps("pnpm", Some("blockExoticSubdeps=true\n"));
        assert_eq!(f.status, TrafficLight::Green);
    }

    #[test]
    fn test_evaluate_exotic_subdeps_with_spaces() {
        let f = evaluate_exotic_subdeps("pnpm", Some("blockExoticSubdeps = true\n"));
        assert_eq!(f.status, TrafficLight::Green);
    }

    #[test]
    fn test_evaluate_exotic_subdeps_not_set() {
        let f = evaluate_exotic_subdeps("pnpm", None);
        assert_eq!(f.status, TrafficLight::Yellow);
    }

    #[test]
    fn test_evaluate_exotic_subdeps_false_positive_rejected() {
        // "blockExoticSubdeps" and "true" on separate lines should not match
        let f = evaluate_exotic_subdeps("pnpm", Some("blockExoticSubdeps=false\nother=true\n"));
        assert_eq!(f.status, TrafficLight::Yellow);
    }

    #[test]
    fn test_exotic_subdeps_not_set() {
        let pm = mock_pm("pnpm");
        let finding = check_exotic_subdeps(&pm);
        assert_eq!(finding.status, TrafficLight::Yellow);
    }

    // --- non-JS ecosystem tests ---

    #[test]
    fn test_pm_hardening_cargo_not_applicable() {
        let pm = mock_pm("cargo");
        let findings = check_pm_hardening(&pm);
        assert_eq!(findings.len(), 2);
        assert!(findings.iter().all(|f| f.status == TrafficLight::Green));
        assert!(findings.iter().all(|f| f.detail.contains("Not applicable")));
    }

    #[test]
    fn test_pm_hardening_go_not_applicable() {
        let pm = mock_pm("go");
        let findings = check_pm_hardening(&pm);
        assert_eq!(findings.len(), 2);
        assert!(findings.iter().all(|f| f.status == TrafficLight::Green));
    }

    // --- npmrc parsing tests ---

    #[test]
    fn test_npmrc_has_value_basic() {
        assert!(npmrc_has_value("key=value\n", "key", "value"));
    }

    #[test]
    fn test_npmrc_has_value_with_spaces() {
        assert!(npmrc_has_value("key = value\n", "key", "value"));
    }

    #[test]
    fn test_npmrc_has_value_comment() {
        assert!(!npmrc_has_value("# key=value\n", "key", "value"));
    }

    #[test]
    fn test_npmrc_has_value_wrong_key() {
        assert!(!npmrc_has_value("other=value\n", "key", "value"));
    }

    #[test]
    fn test_npmrc_has_value_wrong_value() {
        assert!(!npmrc_has_value("key=other\n", "key", "value"));
    }
}
