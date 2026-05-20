use std::process::{Child, Command};

#[derive(Debug, Clone)]
pub enum Harness {
    Claude,
    Codex,
    OpenCode,
}

impl Harness {
    pub fn name(&self) -> &str {
        match self {
            Harness::Claude => "claude",
            Harness::Codex => "codex",
            Harness::OpenCode => "opencode",
        }
    }

    fn prompt_flag(&self) -> &str {
        match self {
            Harness::Claude => "-p",
            Harness::Codex => "-q",
            Harness::OpenCode => "-p",
        }
    }
}

impl std::fmt::Display for Harness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

pub fn detect_harness() -> Option<Harness> {
    detect_harness_in_path(&std::env::var("PATH").unwrap_or_default())
}

fn detect_harness_in_path(path_var: &str) -> Option<Harness> {
    let candidates = [
        ("claude", Harness::Claude),
        ("codex", Harness::Codex),
        ("opencode", Harness::OpenCode),
    ];

    for (bin_name, harness) in &candidates {
        for dir in path_var.split(':') {
            let candidate = std::path::Path::new(dir).join(bin_name);
            if candidate.exists() {
                return Some(harness.clone());
            }
        }
    }

    None
}

pub fn launch(
    harness: &Harness,
    prompt: &str,
    working_dir: &std::path::Path,
) -> Result<Child, std::io::Error> {
    Command::new("open")
        .arg("-a")
        .arg("Terminal.app")
        .arg("--args")
        .arg(harness.name())
        .arg(harness.prompt_flag())
        .arg(prompt)
        .current_dir(working_dir)
        .spawn()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_harness_finds_claude() {
        let temp = tempfile::tempdir().expect("tempdir");
        let claude_path = temp.path().join("claude");
        std::fs::write(&claude_path, "#!/bin/sh\n").expect("write");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&claude_path, std::fs::Permissions::from_mode(0o755))
                .expect("chmod");
        }

        let result = detect_harness_in_path(temp.path().to_str().expect("path"));
        assert!(result.is_some());
        assert!(matches!(result.expect("checked"), Harness::Claude));
    }

    #[test]
    fn test_detect_harness_empty_path() {
        let result = detect_harness_in_path("");
        assert!(result.is_none());
    }

    #[test]
    fn test_harness_display() {
        assert_eq!(Harness::Claude.to_string(), "claude");
        assert_eq!(Harness::Codex.to_string(), "codex");
        assert_eq!(Harness::OpenCode.to_string(), "opencode");
    }
}
