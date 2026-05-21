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
        return HardenFinding {
            name: "npmrc permissions".to_string(),
            status: TrafficLight::Green,
            detail: "No global .npmrc found".to_string(),
            fix_command: None,
            pm: String::new(),
        };
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        match std::fs::metadata(&npmrc) {
            Ok(meta) => {
                let mode = meta.permissions().mode() & 0o777;
                if mode & 0o077 == 0 {
                    HardenFinding {
                        name: "npmrc permissions".to_string(),
                        status: TrafficLight::Green,
                        detail: format!(".npmrc mode is {:o} (owner-only)", mode),
                        fix_command: None,
                        pm: String::new(),
                    }
                } else {
                    HardenFinding {
                        name: "npmrc permissions".to_string(),
                        status: TrafficLight::Red,
                        detail: format!(".npmrc mode is {:o} — readable by group/world", mode),
                        fix_command: Some("chmod 600 ~/.npmrc".to_string()),
                        pm: String::new(),
                    }
                }
            }
            Err(_) => HardenFinding {
                name: "npmrc permissions".to_string(),
                status: TrafficLight::Yellow,
                detail: "Could not read .npmrc metadata".to_string(),
                fix_command: None,
                pm: String::new(),
            },
        }
    }

    #[cfg(not(unix))]
    {
        HardenFinding {
            name: "npmrc permissions".to_string(),
            status: TrafficLight::Yellow,
            detail: "File permission check not supported on this platform".to_string(),
            fix_command: None,
            pm: String::new(),
        }
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

    if exposed.is_empty() {
        HardenFinding {
            name: "Token env vars".to_string(),
            status: TrafficLight::Green,
            detail: "No sensitive token env vars set in shell".to_string(),
            fix_command: None,
            pm: String::new(),
        }
    } else {
        HardenFinding {
            name: "Token env vars".to_string(),
            status: TrafficLight::Yellow,
            detail: format!("Tokens found in env: {}", exposed.join(", ")),
            fix_command: Some(
                "Use a credential manager instead of shell env vars for tokens".to_string(),
            ),
            pm: String::new(),
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
    }

    #[test]
    fn test_token_env_vars_returns_finding() {
        let finding = check_token_env_vars();
        assert!(finding.status == TrafficLight::Green || finding.status == TrafficLight::Yellow);
        assert_eq!(finding.name, "Token env vars");
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
        };
        let findings = check_credential_hygiene(dir.path(), &[pm]);
        assert!(findings.iter().all(|f| f.status != TrafficLight::Red));
    }
}
