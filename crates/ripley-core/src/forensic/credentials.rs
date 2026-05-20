use std::path::{Path, PathBuf};

use crate::types::Severity;

use super::ioc::IocProfile;

#[derive(Debug, Clone)]
pub struct CredentialFinding {
    pub path: PathBuf,
    pub description: String,
    pub severity: Severity,
    pub rotation_command: Option<String>,
    pub profile_id: String,
}

#[derive(Debug, Clone)]
pub struct ExposureReport {
    pub findings: Vec<CredentialFinding>,
    pub dead_man_switch_warning: Option<String>,
}

pub fn assess_exposure(profiles: &[IocProfile], home: &Path) -> ExposureReport {
    let mut findings = Vec::new();
    let mut dead_man_switch_warning = None;

    for profile in profiles {
        if profile.credentials.dead_man_switch {
            let warning = profile
                .credentials
                .rotation_warning
                .clone()
                .unwrap_or_else(|| {
                    format!(
                        "WARNING: {} installs a dead man switch. \
                         Back up your home directory before rotating any credentials.",
                        profile.name
                    )
                });
            dead_man_switch_warning = Some(warning);
        }

        for target in &profile.credentials.targeted {
            let expanded = expand_cred_path(target, home);
            let matched_paths = resolve_cred_glob(&expanded);

            for matched in matched_paths {
                if matched.exists() {
                    let rotation = rotation_command_for(&matched);
                    findings.push(CredentialFinding {
                        path: matched,
                        description: format!("Credential at risk ({}): {}", profile.name, target),
                        severity: if profile.credentials.dead_man_switch {
                            Severity::Critical
                        } else {
                            Severity::High
                        },
                        rotation_command: rotation,
                        profile_id: profile.id.clone(),
                    });
                }
            }
        }
    }

    ExposureReport {
        findings,
        dead_man_switch_warning,
    }
}

fn expand_cred_path(pattern: &str, home: &Path) -> String {
    if pattern.starts_with("~/") {
        return format!("{}{}", home.display(), &pattern[1..]);
    }
    pattern.to_string()
}

fn resolve_cred_glob(pattern: &str) -> Vec<PathBuf> {
    if !pattern.contains('*') {
        return vec![PathBuf::from(pattern)];
    }

    let path = Path::new(pattern);
    let parent = match path.parent() {
        Some(p) => p,
        None => return vec![],
    };
    let file_pattern = match path.file_name().and_then(|n| n.to_str()) {
        Some(p) => p,
        None => return vec![],
    };

    let regex_pattern = glob_to_regex(file_pattern);
    let re = match regex::Regex::new(&regex_pattern) {
        Ok(r) => r,
        Err(_) => return vec![],
    };

    let entries = match std::fs::read_dir(parent) {
        Ok(e) => e,
        Err(_) => return vec![],
    };

    entries
        .flatten()
        .filter(|entry| {
            entry
                .file_name()
                .to_str()
                .is_some_and(|name| re.is_match(name))
        })
        .map(|entry| entry.path())
        .collect()
}

fn glob_to_regex(pattern: &str) -> String {
    let mut regex = String::from("^");
    for ch in pattern.chars() {
        match ch {
            '*' => regex.push_str(".*"),
            '?' => regex.push('.'),
            '.' | '(' | ')' | '+' | '|' | '^' | '$' | '@' | '{' | '}' | '[' | ']' => {
                regex.push('\\');
                regex.push(ch);
            }
            _ => regex.push(ch),
        }
    }
    regex.push('$');
    regex
}

fn rotation_command_for(path: &Path) -> Option<String> {
    let path_str = path.to_str().unwrap_or("");
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    if path_str.contains(".npmrc") {
        Some("npm token revoke <token> && npm login".into())
    } else if path_str.contains(".ssh/") {
        if file_name == "authorized_keys" {
            Some("Review and remove unauthorized keys from ~/.ssh/authorized_keys".into())
        } else {
            Some("Generate new SSH key: ssh-keygen -t ed25519 && update remote services".into())
        }
    } else if path_str.contains(".aws/credentials") {
        Some("aws iam delete-access-key && aws iam create-access-key".into())
    } else if path_str.contains(".gitconfig") {
        Some("Review ~/.gitconfig for unauthorized credential helpers or URLs".into())
    } else if file_name == ".env" || path_str.contains(".env") {
        Some("Rotate all secrets in .env and redeploy".into())
    } else if path_str.contains(".docker/config.json") {
        Some("docker logout && docker login".into())
    } else if path_str.contains(".kube/config") {
        Some("Rotate Kubernetes service account tokens and kubeconfig credentials".into())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forensic::ioc::{IocCredentials, IocIndicators, IocProfile};

    fn profile_with_creds(id: &str, targeted: Vec<String>, dead_man_switch: bool) -> IocProfile {
        IocProfile {
            id: id.into(),
            name: format!("Test Profile {id}"),
            description: "test".into(),
            date: "2026-05-20".into(),
            references: vec![],
            packages: None,
            indicators: IocIndicators::default(),
            credentials: IocCredentials {
                targeted,
                dead_man_switch,
                rotation_warning: if dead_man_switch {
                    Some("Back up before rotating!".into())
                } else {
                    None
                },
            },
        }
    }

    #[test]
    fn test_assess_exposure_finds_existing_creds() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            dir.path().join(".npmrc"),
            "//registry.npmjs.org/:_authToken=secret",
        )
        .expect("write");

        let profile = profile_with_creds("test-1", vec!["~/.npmrc".into()], false);
        let report = assess_exposure(&[profile], dir.path());

        assert_eq!(report.findings.len(), 1);
        assert!(report.findings[0].path.ends_with(".npmrc"));
        assert!(report.findings[0].rotation_command.is_some());
        assert_eq!(report.findings[0].severity, Severity::High);
        assert!(report.dead_man_switch_warning.is_none());
    }

    #[test]
    fn test_assess_exposure_dead_man_switch() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join(".npmrc"), "token").expect("write");

        let profile = profile_with_creds("dms-test", vec!["~/.npmrc".into()], true);
        let report = assess_exposure(&[profile], dir.path());

        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].severity, Severity::Critical);
        assert!(report.dead_man_switch_warning.is_some());
        assert!(
            report
                .dead_man_switch_warning
                .as_ref()
                .expect("checked")
                .contains("Back up")
        );
    }

    #[test]
    fn test_assess_exposure_no_files() {
        let dir = tempfile::tempdir().expect("tempdir");

        let profile = profile_with_creds(
            "empty-test",
            vec!["~/.npmrc".into(), "~/.aws/credentials".into()],
            false,
        );
        let report = assess_exposure(&[profile], dir.path());
        assert!(report.findings.is_empty());
    }

    #[test]
    fn test_assess_exposure_glob_ssh_keys() {
        let dir = tempfile::tempdir().expect("tempdir");
        let ssh_dir = dir.path().join(".ssh");
        std::fs::create_dir_all(&ssh_dir).expect("mkdir");
        std::fs::write(ssh_dir.join("id_rsa"), "private key").expect("write");
        std::fs::write(ssh_dir.join("id_ed25519"), "private key").expect("write");

        let profile = profile_with_creds("ssh-test", vec!["~/.ssh/id_*".into()], false);
        let report = assess_exposure(&[profile], dir.path());

        assert_eq!(report.findings.len(), 2);
        assert!(report.findings.iter().all(|f| f.rotation_command.is_some()));
    }

    #[test]
    fn test_assess_exposure_multiple_profiles() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join(".npmrc"), "token").expect("write");
        std::fs::write(dir.path().join(".gitconfig"), "[user]").expect("write");

        let p1 = profile_with_creds("p1", vec!["~/.npmrc".into()], false);
        let p2 = profile_with_creds("p2", vec!["~/.gitconfig".into()], false);
        let report = assess_exposure(&[p1, p2], dir.path());

        assert_eq!(report.findings.len(), 2);
        assert!(report.findings.iter().any(|f| f.profile_id == "p1"));
        assert!(report.findings.iter().any(|f| f.profile_id == "p2"));
    }

    #[test]
    fn test_rotation_commands() {
        assert!(
            rotation_command_for(Path::new("/home/user/.npmrc"))
                .expect("has command")
                .contains("npm")
        );

        assert!(
            rotation_command_for(Path::new("/home/user/.ssh/id_rsa"))
                .expect("has command")
                .contains("ssh-keygen")
        );

        assert!(
            rotation_command_for(Path::new("/home/user/.ssh/authorized_keys"))
                .expect("has command")
                .contains("unauthorized")
        );

        assert!(
            rotation_command_for(Path::new("/home/user/.aws/credentials"))
                .expect("has command")
                .contains("aws")
        );

        assert!(
            rotation_command_for(Path::new("/home/user/.gitconfig"))
                .expect("has command")
                .contains("gitconfig")
        );

        assert!(
            rotation_command_for(Path::new("/home/user/project/.env"))
                .expect("has command")
                .contains("Rotate")
        );

        assert!(rotation_command_for(Path::new("/random/file.txt")).is_none());
    }

    #[test]
    fn test_empty_profiles() {
        let dir = tempfile::tempdir().expect("tempdir");
        let report = assess_exposure(&[], dir.path());
        assert!(report.findings.is_empty());
        assert!(report.dead_man_switch_warning.is_none());
    }
}
