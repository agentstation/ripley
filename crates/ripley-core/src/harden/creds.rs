use std::path::Path;

use crate::audit::TrafficLight;

use super::{DetectedPm, HardenFinding};

pub fn check_credential_hygiene(home: &Path, pms: &[DetectedPm]) -> Vec<HardenFinding> {
    let mut findings = Vec::new();

    findings.push(check_npmrc_permissions(home));
    findings.push(check_token_env_vars());

    for pm in pms {
        if let Some(ref config_path) = pm.config_path {
            findings.push(check_project_npmrc_tokens(config_path, &pm.name));
        }
    }

    findings
}

pub fn check_npmrc_permissions(home: &Path) -> HardenFinding {
    let npmrc = home.join(".npmrc");

    if !npmrc.exists() {
        return evaluate_npmrc_permissions(None);
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        match std::fs::metadata(&npmrc) {
            Ok(meta) => {
                let mode = meta.permissions().mode() & 0o777;
                evaluate_npmrc_permissions(Some(mode))
            }
            Err(_) => evaluate_npmrc_permissions(Some(0xFFFF)),
        }
    }

    #[cfg(not(unix))]
    {
        HardenFinding {
            name: "npmrc permissions".to_string(),
            status: TrafficLight::Yellow,
            detail: "File permission check not supported on this platform".to_string(),
            fix_command: None,
            pm: "all".to_string(),
        }
    }
}

/// Evaluate npmrc permissions. `None` means no .npmrc exists. `Some(mode)` is the
/// octal permission bits (only lower 12 bits matter). A sentinel value of `0xFFFF`
/// indicates a metadata read failure.
pub fn evaluate_npmrc_permissions(mode: Option<u32>) -> HardenFinding {
    match mode {
        None => HardenFinding {
            name: "npmrc permissions".to_string(),
            status: TrafficLight::Green,
            detail: "No global .npmrc found".to_string(),
            fix_command: None,
            pm: "all".to_string(),
        },
        Some(0xFFFF) => HardenFinding {
            name: "npmrc permissions".to_string(),
            status: TrafficLight::Yellow,
            detail: "Could not read .npmrc metadata".to_string(),
            fix_command: None,
            pm: "all".to_string(),
        },
        Some(mode) if mode & 0o077 == 0 => HardenFinding {
            name: "npmrc permissions".to_string(),
            status: TrafficLight::Green,
            detail: format!(".npmrc mode is {mode:o} (owner-only)"),
            fix_command: None,
            pm: "all".to_string(),
        },
        Some(mode) => HardenFinding {
            name: "npmrc permissions".to_string(),
            status: TrafficLight::Red,
            detail: format!(".npmrc mode is {mode:o} — readable by group/world"),
            fix_command: Some("chmod 600 ~/.npmrc".to_string()),
            pm: "all".to_string(),
        },
    }
}

pub fn check_token_env_vars() -> HardenFinding {
    let sensitive_vars = [
        "NPM_TOKEN",
        "NODE_AUTH_TOKEN",
        "GH_TOKEN",
        "GITHUB_TOKEN",
        "PYPI_TOKEN",
    ];

    let exposed: Vec<&str> = sensitive_vars
        .iter()
        .filter(|var| std::env::var(var).is_ok())
        .copied()
        .collect();

    evaluate_token_env_vars(&exposed)
}

pub fn evaluate_token_env_vars(exposed: &[&str]) -> HardenFinding {
    if exposed.is_empty() {
        HardenFinding {
            name: "Token env vars".to_string(),
            status: TrafficLight::Green,
            detail: "No sensitive token env vars set in shell".to_string(),
            fix_command: None,
            pm: "all".to_string(),
        }
    } else {
        HardenFinding {
            name: "Token env vars".to_string(),
            status: TrafficLight::Yellow,
            detail: format!("Tokens found in env: {}", exposed.join(", ")),
            fix_command: Some(
                "Use a credential manager instead of shell env vars for tokens".to_string(),
            ),
            pm: "all".to_string(),
        }
    }
}

pub fn check_project_npmrc_tokens(config_path: &Path, pm_name: &str) -> HardenFinding {
    let content = match std::fs::read_to_string(config_path) {
        Ok(c) => c,
        Err(_) => {
            return HardenFinding {
                name: "Project .npmrc tokens".to_string(),
                status: TrafficLight::Yellow,
                detail: "Could not read project .npmrc".to_string(),
                fix_command: None,
                pm: pm_name.to_string(),
            };
        }
    };

    evaluate_project_npmrc_tokens(&content, pm_name)
}

pub fn evaluate_project_npmrc_tokens(content: &str, pm_name: &str) -> HardenFinding {
    let has_hardcoded_token = content.contains("_authToken=npm_")
        || content.contains("_auth=")
        || content.contains("_authToken=ghp_");

    if has_hardcoded_token {
        HardenFinding {
            name: "Project .npmrc tokens".to_string(),
            status: TrafficLight::Red,
            detail: "Hardcoded token found in project .npmrc".to_string(),
            fix_command: Some("Replace with ${NPM_TOKEN} env var reference in .npmrc".to_string()),
            pm: pm_name.to_string(),
        }
    } else if content.contains("_authToken") {
        HardenFinding {
            name: "Project .npmrc tokens".to_string(),
            status: TrafficLight::Green,
            detail: "Token reference uses env var substitution".to_string(),
            fix_command: None,
            pm: pm_name.to_string(),
        }
    } else {
        HardenFinding {
            name: "Project .npmrc tokens".to_string(),
            status: TrafficLight::Green,
            detail: "No tokens in project .npmrc".to_string(),
            fix_command: None,
            pm: pm_name.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_npmrc_not_present() {
        let dir = tempfile::tempdir().expect("tempdir");
        let finding = check_npmrc_permissions(dir.path());
        assert_eq!(finding.status, TrafficLight::Green);
        assert_eq!(finding.pm, "all");
    }

    #[cfg(unix)]
    #[test]
    fn test_npmrc_restrictive_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().expect("tempdir");
        let npmrc = dir.path().join(".npmrc");
        std::fs::write(&npmrc, "registry=https://registry.npmjs.org/\n").expect("write");
        std::fs::set_permissions(&npmrc, std::fs::Permissions::from_mode(0o600)).expect("chmod");
        let finding = check_npmrc_permissions(dir.path());
        assert_eq!(finding.status, TrafficLight::Green);
        assert_eq!(finding.pm, "all");
    }

    #[cfg(unix)]
    #[test]
    fn test_npmrc_world_readable() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().expect("tempdir");
        let npmrc = dir.path().join(".npmrc");
        std::fs::write(&npmrc, "//registry.npmjs.org/:_authToken=npm_abc\n").expect("write");
        std::fs::set_permissions(&npmrc, std::fs::Permissions::from_mode(0o644)).expect("chmod");
        let finding = check_npmrc_permissions(dir.path());
        assert_eq!(finding.status, TrafficLight::Red);
        assert_eq!(finding.pm, "all");
    }

    // --- evaluate tests for npmrc permissions ---

    #[test]
    fn test_evaluate_npmrc_no_file() {
        let f = evaluate_npmrc_permissions(None);
        assert_eq!(f.status, TrafficLight::Green);
        assert_eq!(f.pm, "all");
    }

    #[test]
    fn test_evaluate_npmrc_owner_only() {
        let f = evaluate_npmrc_permissions(Some(0o600));
        assert_eq!(f.status, TrafficLight::Green);
    }

    #[test]
    fn test_evaluate_npmrc_world_readable() {
        let f = evaluate_npmrc_permissions(Some(0o644));
        assert_eq!(f.status, TrafficLight::Red);
        assert!(f.fix_command.is_some());
    }

    #[test]
    fn test_evaluate_npmrc_metadata_error() {
        let f = evaluate_npmrc_permissions(Some(0xFFFF));
        assert_eq!(f.status, TrafficLight::Yellow);
    }

    // --- token env var tests ---

    #[test]
    fn test_token_env_vars_returns_finding() {
        let finding = check_token_env_vars();
        assert!(!finding.name.is_empty());
        assert_eq!(finding.name, "Token env vars");
        assert!(finding.status == TrafficLight::Green || finding.status == TrafficLight::Yellow);
        assert_eq!(finding.pm, "all");
    }

    #[test]
    fn test_evaluate_token_env_vars_none_exposed() {
        let f = evaluate_token_env_vars(&[]);
        assert_eq!(f.status, TrafficLight::Green);
        assert_eq!(f.pm, "all");
    }

    #[test]
    fn test_evaluate_token_env_vars_some_exposed() {
        let f = evaluate_token_env_vars(&["NPM_TOKEN", "GH_TOKEN"]);
        assert_eq!(f.status, TrafficLight::Yellow);
        assert!(f.detail.contains("NPM_TOKEN"));
        assert!(f.detail.contains("GH_TOKEN"));
        assert_eq!(f.pm, "all");
    }

    // --- project npmrc token evaluate tests ---

    #[test]
    fn test_evaluate_project_npmrc_hardcoded() {
        let f = evaluate_project_npmrc_tokens(
            "//registry.npmjs.org/:_authToken=npm_secrettoken123\n",
            "npm",
        );
        assert_eq!(f.status, TrafficLight::Red);
    }

    #[test]
    fn test_evaluate_project_npmrc_env_ref() {
        let f =
            evaluate_project_npmrc_tokens("//registry.npmjs.org/:_authToken=${NPM_TOKEN}\n", "npm");
        assert_eq!(f.status, TrafficLight::Green);
    }

    #[test]
    fn test_evaluate_project_npmrc_no_tokens() {
        let f = evaluate_project_npmrc_tokens("registry=https://registry.npmjs.org/\n", "npm");
        assert_eq!(f.status, TrafficLight::Green);
    }

    #[test]
    fn test_project_npmrc_hardcoded_token() {
        let dir = tempfile::tempdir().expect("tempdir");
        let npmrc = dir.path().join(".npmrc");
        std::fs::write(
            &npmrc,
            "//registry.npmjs.org/:_authToken=npm_secrettoken123\n",
        )
        .expect("write");
        let finding = check_project_npmrc_tokens(&npmrc, "npm");
        assert_eq!(finding.status, TrafficLight::Red);
    }

    #[test]
    fn test_project_npmrc_env_reference() {
        let dir = tempfile::tempdir().expect("tempdir");
        let npmrc = dir.path().join(".npmrc");
        std::fs::write(&npmrc, "//registry.npmjs.org/:_authToken=${NPM_TOKEN}\n").expect("write");
        let finding = check_project_npmrc_tokens(&npmrc, "npm");
        assert_eq!(finding.status, TrafficLight::Green);
    }

    #[test]
    fn test_project_npmrc_no_tokens() {
        let dir = tempfile::tempdir().expect("tempdir");
        let npmrc = dir.path().join(".npmrc");
        std::fs::write(&npmrc, "registry=https://registry.npmjs.org/\n").expect("write");
        let finding = check_project_npmrc_tokens(&npmrc, "npm");
        assert_eq!(finding.status, TrafficLight::Green);
    }

    #[test]
    fn test_check_credential_hygiene_empty() {
        let dir = tempfile::tempdir().expect("tempdir");
        let pms: Vec<DetectedPm> = vec![];
        let findings = check_credential_hygiene(dir.path(), &pms);
        assert!(!findings.is_empty());
        // All findings should have a non-empty pm
        assert!(findings.iter().all(|f| !f.pm.is_empty()));
    }

    #[cfg(unix)]
    #[test]
    fn test_project_npmrc_with_pm_config() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().expect("tempdir");
        let global_npmrc = dir.path().join(".npmrc");
        std::fs::write(&global_npmrc, "registry=https://registry.npmjs.org/\n").expect("write");
        std::fs::set_permissions(&global_npmrc, std::fs::Permissions::from_mode(0o600))
            .expect("chmod");

        let project_npmrc = dir.path().join("project.npmrc");
        std::fs::write(&project_npmrc, "save-exact=true\n").expect("write");

        let pm = DetectedPm {
            name: "npm".to_string(),
            version: Some("10.0.0".to_string()),
            lockfile_path: PathBuf::from("package-lock.json"),
            config_path: Some(project_npmrc),
            binary: false,
        };
        let findings = check_credential_hygiene(dir.path(), &[pm]);
        assert!(findings.iter().all(|f| f.status != TrafficLight::Red));
    }
}
