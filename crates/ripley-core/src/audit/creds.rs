use std::io::BufRead;
use std::path::Path;

use super::{AuditCategory, AuditFinding, CategoryReport, TrafficLight};

const TOKEN_PATTERNS: &[(&str, &str)] = &[
    ("npm_", "npm token"),
    ("ghp_", "GitHub personal access token"),
    ("gho_", "GitHub OAuth token"),
    ("ghs_", "GitHub App installation token"),
    ("AKIA", "AWS access key"),
    ("xoxb-", "Slack bot token"),
    ("xoxp-", "Slack user token"),
    ("glpat-", "GitLab personal access token"),
    ("pypi-AgEIcH", "PyPI token"),
];

/// Check if a line contains an `sk-` token at a word boundary followed by at
/// least 20 alphanumeric characters (avoids false positives like `flask-app`).
fn contains_sk_token(line: &str) -> bool {
    let pat = "sk-";
    let mut start = 0;
    while let Some(pos) = line[start..].find(pat) {
        let abs = start + pos;
        // Check word boundary: must be preceded by start-of-string, space, =, ", or '
        let at_boundary = abs == 0 || {
            let prev = line.as_bytes()[abs - 1];
            matches!(prev, b' ' | b'=' | b'"' | b'\'' | b'\t' | b'\n')
        };
        if at_boundary {
            let after = &line[abs + pat.len()..];
            let alnum_count = after.chars().take_while(|c| c.is_alphanumeric()).count();
            if alnum_count >= 20 {
                return true;
            }
        }
        start = abs + 1;
    }
    false
}

/// Shell-quote a string using single quotes with proper escaping.
fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

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

    // Fix 3: Check sk- tokens with word-boundary matching to avoid false positives
    if content.lines().any(contains_sk_token) {
        findings.push(AuditFinding {
            name: "Shell history: API secret key".to_string(),
            status: TrafficLight::Red,
            detail: "Found API secret key pattern in shell history".to_string(),
            fix_command: Some(
                "Remove the token from history and rotate the API secret key".to_string(),
            ),
        });
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

// Fix 4: Split evaluate_env_in_git into collect (I/O) and evaluate (pure)
pub fn collect_env_in_git(project_path: &Path) -> Result<String, std::io::Error> {
    let output = std::process::Command::new("git")
        .args(["ls-files", ".env", ".env.local", ".env.production"])
        .current_dir(project_path)
        .output()?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn evaluate_env_in_git(git_output: &str) -> Vec<AuditFinding> {
    let mut findings = Vec::new();

    let tracked_files: Vec<&str> = git_output
        .lines()
        .filter(|l| !l.trim().is_empty())
        .collect();

    if tracked_files.is_empty() {
        findings.push(AuditFinding {
            name: ".env files in git".to_string(),
            status: TrafficLight::Green,
            detail: "No .env files tracked by git".to_string(),
            fix_command: None,
        });
    } else {
        for file in tracked_files {
            // Fix 2: Shell-quote filenames to prevent command injection
            let quoted = shell_quote(file);
            findings.push(AuditFinding {
                name: format!(".env in git: {file}"),
                status: TrafficLight::Red,
                detail: format!("{file} is committed to git"),
                fix_command: Some(format!(
                    "git rm --cached {quoted} && echo {quoted} >> .gitignore"
                )),
            });
        }
    }

    findings
}

/// Combined check for backwards compatibility with `check_credentials`.
fn check_env_in_git(project_path: &Path) -> Vec<AuditFinding> {
    match collect_env_in_git(project_path) {
        Ok(output) => evaluate_env_in_git(&output),
        Err(_) => vec![AuditFinding {
            name: ".env files in git".to_string(),
            status: TrafficLight::Yellow,
            detail: "Could not check git status (not a git repository?)".to_string(),
            fix_command: None,
        }],
    }
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

    // Fix 3: Check sk- tokens with word-boundary matching
    if content.lines().any(contains_sk_token) {
        findings.push(AuditFinding {
            name: format!("RC file token: {filename}"),
            status: TrafficLight::Red,
            detail: format!("Found API secret key in {filename}"),
            fix_command: Some(format!(
                "Remove the token from {filename} and use a credential manager"
            )),
        });
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

    // Fix 10: Use BufReader for shell history files instead of read_to_string
    for hist_name in &[".bash_history", ".zsh_history"] {
        let hist_path = home.join(hist_name);
        if let Ok(file) = std::fs::File::open(&hist_path) {
            let reader = std::io::BufReader::new(file);
            let mut content = String::new();
            for line in reader.lines() {
                match line {
                    Ok(l) => {
                        content.push_str(&l);
                        content.push('\n');
                    }
                    Err(_) => break,
                }
            }
            findings.extend(evaluate_shell_history_secrets(&content));
        }
    }

    let cwd = std::env::current_dir().unwrap_or_else(|_| home.to_path_buf());
    findings.extend(check_env_in_git(&cwd));

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

    #[test]
    fn test_evaluate_env_in_git_tracked() {
        let output = ".env\n.env.local\n";
        let findings = evaluate_env_in_git(output);
        assert_eq!(findings.len(), 2);
        assert!(findings.iter().all(|f| f.status == TrafficLight::Red));
        // Verify shell quoting in fix_command
        assert!(
            findings[0]
                .fix_command
                .as_ref()
                .is_some_and(|c| c.contains("'.env'"))
        );
    }

    #[test]
    fn test_evaluate_env_in_git_clean() {
        let output = "";
        let findings = evaluate_env_in_git(output);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].status, TrafficLight::Green);
    }

    #[test]
    fn test_evaluate_env_in_git_shell_quote_special() {
        let output = ".env'special\n";
        let findings = evaluate_env_in_git(output);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].status, TrafficLight::Red);
        // Single-quote escaping should replace ' with '\''
        let cmd = findings[0].fix_command.as_ref().expect("fix_command");
        assert!(cmd.contains("'\\''"));
    }

    #[test]
    fn test_sk_token_real() {
        assert!(contains_sk_token(
            "export OPENAI_KEY=sk-abcdefghijklmnopqrstuvwxyz1234"
        ));
    }

    #[test]
    fn test_sk_token_at_start() {
        assert!(contains_sk_token("sk-abcdefghijklmnopqrstuvwxyz1234"));
    }

    #[test]
    fn test_sk_token_false_positive_flask() {
        assert!(!contains_sk_token("pip install flask-app"));
    }

    #[test]
    fn test_sk_token_false_positive_desk() {
        assert!(!contains_sk_token("buy a desk-lamp"));
    }

    #[test]
    fn test_sk_token_short_suffix() {
        // sk- followed by fewer than 20 chars should not match
        assert!(!contains_sk_token("sk-short"));
    }
}
