use std::path::Path;

use super::{AuditCategory, AuditFinding, CategoryReport, TrafficLight};

const TOKEN_PATTERNS: &[(&str, &str)] = &[
    ("npm_", "npm token"),
    ("ghp_", "GitHub personal access token"),
    ("gho_", "GitHub OAuth token"),
    ("ghs_", "GitHub App installation token"),
    ("sk-", "API secret key"),
    ("AKIA", "AWS access key"),
    ("xoxb-", "Slack bot token"),
    ("xoxp-", "Slack user token"),
    ("glpat-", "GitLab personal access token"),
    ("pypi-AgEIcH", "PyPI token"),
];

pub fn evaluate_shell_history_secrets(content: &str) -> Vec<AuditFinding> {
    let mut findings = Vec::new();

    for (pattern, description) in TOKEN_PATTERNS {
        if content.contains(pattern) {
            findings.push(AuditFinding {
                name: format!("Shell history: {description}"),
                status: TrafficLight::Red,
                detail: format!("Found {description} pattern in shell history"),
                fix_command: Some(format!(
                    "Remove the token from history and rotate the {description}"
                )),
            });
        }
    }

    if findings.is_empty() {
        findings.push(AuditFinding {
            name: "Shell history secrets".to_string(),
            status: TrafficLight::Green,
            detail: "No token patterns found in shell history".to_string(),
            fix_command: None,
        });
    }

    findings
}

pub fn evaluate_env_in_git(project_path: &Path) -> Vec<AuditFinding> {
    let mut findings = Vec::new();

    let output = std::process::Command::new("git")
        .args(["ls-files", ".env", ".env.local", ".env.production"])
        .current_dir(project_path)
        .output();

    match output {
        Ok(out) => {
            let tracked = String::from_utf8_lossy(&out.stdout);
            let tracked_files: Vec<&str> =
                tracked.lines().filter(|l| !l.trim().is_empty()).collect();

            if tracked_files.is_empty() {
                findings.push(AuditFinding {
                    name: ".env files in git".to_string(),
                    status: TrafficLight::Green,
                    detail: "No .env files tracked by git".to_string(),
                    fix_command: None,
                });
            } else {
                for file in tracked_files {
                    findings.push(AuditFinding {
                        name: format!(".env in git: {file}"),
                        status: TrafficLight::Red,
                        detail: format!("{file} is committed to git"),
                        fix_command: Some(format!(
                            "git rm --cached {file} && echo '{file}' >> .gitignore"
                        )),
                    });
                }
            }
        }
        Err(_) => {
            findings.push(AuditFinding {
                name: ".env files in git".to_string(),
                status: TrafficLight::Yellow,
                detail: "Could not check git status (not a git repository?)".to_string(),
                fix_command: None,
            });
        }
    }

    findings
}

pub fn evaluate_npmrc_tokens(content: &str) -> Vec<AuditFinding> {
    let mut findings = Vec::new();

    let has_token = content.contains("_authToken") || content.contains("_auth=");

    if !has_token {
        findings.push(AuditFinding {
            name: "npm tokens".to_string(),
            status: TrafficLight::Green,
            detail: "No tokens found in .npmrc".to_string(),
            fix_command: None,
        });
        return findings;
    }

    let is_scoped = content
        .lines()
        .filter(|l| l.contains("_authToken") || l.contains("_auth="))
        .all(|l| l.contains("@") && l.contains("/:_authToken"));

    if is_scoped {
        findings.push(AuditFinding {
            name: "npm tokens".to_string(),
            status: TrafficLight::Yellow,
            detail: "npm token found (scoped)".to_string(),
            fix_command: Some("Verify token has minimal scope and expiry set".to_string()),
        });
    } else {
        findings.push(AuditFinding {
            name: "npm tokens".to_string(),
            status: TrafficLight::Red,
            detail: "npm token found (broadly scoped, no expiry detected)".to_string(),
            fix_command: Some("npm token revoke <token> && npm login --scope=@yourorg".to_string()),
        });
    }

    findings
}

pub fn evaluate_rc_file_tokens(content: &str, path: &Path) -> Vec<AuditFinding> {
    let filename = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let mut findings = Vec::new();

    for (pattern, description) in TOKEN_PATTERNS {
        if content.contains(pattern) {
            findings.push(AuditFinding {
                name: format!("RC file token: {filename}"),
                status: TrafficLight::Red,
                detail: format!("Found {description} in {filename}"),
                fix_command: Some(format!(
                    "Remove the token from {filename} and use a credential manager"
                )),
            });
        }
    }

    if findings.is_empty() {
        findings.push(AuditFinding {
            name: format!("RC file: {filename}"),
            status: TrafficLight::Green,
            detail: "No token patterns found".to_string(),
            fix_command: None,
        });
    }

    findings
}

pub fn check_credentials(home: &Path) -> CategoryReport {
    let mut findings = Vec::new();

    for hist_name in &[".bash_history", ".zsh_history"] {
        let hist_path = home.join(hist_name);
        if let Ok(content) = std::fs::read_to_string(&hist_path) {
            findings.extend(evaluate_shell_history_secrets(&content));
        }
    }

    let cwd = std::env::current_dir().unwrap_or_else(|_| home.to_path_buf());
    findings.extend(evaluate_env_in_git(&cwd));

    let npmrc_path = home.join(".npmrc");
    if let Ok(content) = std::fs::read_to_string(&npmrc_path) {
        findings.extend(evaluate_npmrc_tokens(&content));
    }

    for rc_name in &[".bashrc", ".zshrc", ".profile", ".bash_profile"] {
        let rc_path = home.join(rc_name);
        if let Ok(content) = std::fs::read_to_string(&rc_path) {
            findings.extend(evaluate_rc_file_tokens(&content, &rc_path));
        }
    }

    if findings.is_empty() {
        findings.push(AuditFinding {
            name: "Credentials".to_string(),
            status: TrafficLight::Green,
            detail: "No credential issues found".to_string(),
            fix_command: None,
        });
    }

    CategoryReport::new(AuditCategory::Credentials, findings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_history_with_npm_token() {
        let content = "npm publish\nexport NPM_TOKEN=npm_abc123def456\nls";
        let findings = evaluate_shell_history_secrets(content);
        assert!(findings.iter().any(|f| f.status == TrafficLight::Red));
    }

    #[test]
    fn test_evaluate_history_clean() {
        let content = "ls\ncd ~/projects\ngit status\ncargo build";
        let findings = evaluate_shell_history_secrets(content);
        assert!(findings.iter().all(|f| f.status == TrafficLight::Green));
    }

    #[test]
    fn test_evaluate_history_github_token() {
        let content = "curl -H 'Authorization: token ghp_xxxxxxxxxxxx' https://api.github.com";
        let findings = evaluate_shell_history_secrets(content);
        assert!(findings.iter().any(|f| f.status == TrafficLight::Red));
        assert!(findings.iter().any(|f| f.detail.contains("GitHub")));
    }

    #[test]
    fn test_evaluate_npmrc_broad_token() {
        let content = "//registry.npmjs.org/:_authToken=npm_abc123\n";
        let findings = evaluate_npmrc_tokens(content);
        assert!(findings.iter().any(|f| f.status == TrafficLight::Red));
    }

    #[test]
    fn test_evaluate_npmrc_no_token() {
        let content = "registry=https://registry.npmjs.org/\n";
        let findings = evaluate_npmrc_tokens(content);
        assert!(findings.iter().all(|f| f.status == TrafficLight::Green));
    }

    #[test]
    fn test_evaluate_rc_file_clean() {
        let content = "export PATH=\"$HOME/bin:$PATH\"\nalias ll='ls -la'\n";
        let findings = evaluate_rc_file_tokens(content, Path::new(".zshrc"));
        assert!(findings.iter().all(|f| f.status == TrafficLight::Green));
    }

    #[test]
    fn test_evaluate_rc_file_with_token() {
        let content = "export GITHUB_TOKEN=ghp_1234567890abcdef\n";
        let findings = evaluate_rc_file_tokens(content, Path::new(".bashrc"));
        assert!(findings.iter().any(|f| f.status == TrafficLight::Red));
    }

    #[test]
    fn test_evaluate_rc_file_aws_key() {
        let content = "export AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE\n";
        let findings = evaluate_rc_file_tokens(content, Path::new(".profile"));
        assert!(findings.iter().any(|f| f.status == TrafficLight::Red));
    }
}
