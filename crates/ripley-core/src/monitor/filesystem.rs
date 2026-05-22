use std::path::{Path, PathBuf};

use serde::Serialize;

use super::process::{AlertReason, ProcessAlert};
use crate::forensic::network::NetworkConnection;
use crate::platform;
use crate::types::Severity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum FsEventKind {
    Created,
    Modified,
    Deleted,
}

pub fn persistence_watch_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(home) = platform::home_dir() {
        paths.push(home.join(".claude"));
        paths.push(home.join(".vscode"));
        paths.push(home.join(".mcp.json"));
        paths.push(home.join(".cursor"));

        #[cfg(target_os = "macos")]
        {
            paths.push(home.join("Library/LaunchAgents"));
        }

        #[cfg(target_os = "linux")]
        {
            paths.push(home.join(".config/systemd/user"));
            paths.push(home.join(".config/autostart"));
        }

        for rc in &[".bashrc", ".zshrc", ".profile", ".bash_profile"] {
            paths.push(home.join(rc));
        }
    }

    #[cfg(target_os = "linux")]
    {
        paths.push(PathBuf::from("/etc/systemd/system"));
        paths.push(PathBuf::from("/etc/cron.d"));
    }

    paths
}

pub fn evaluate_fs_event(path: &Path, kind: FsEventKind) -> Option<ProcessAlert> {
    if kind == FsEventKind::Deleted {
        return None;
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let dummy_connection = NetworkConnection {
        process: String::new(),
        pid: 0,
        protocol: String::new(),
        remote_addr: String::new(),
        remote_port: 0,
        state: String::new(),
    };

    if is_mcp_config(path) {
        return Some(ProcessAlert {
            connection: dummy_connection,
            reason: AlertReason::McpConfigChange {
                path: path.to_path_buf(),
            },
            severity: Severity::High,
            timestamp: now,
        });
    }

    if is_lockfile_edit(path) {
        return Some(ProcessAlert {
            connection: dummy_connection,
            reason: AlertReason::LockfileEdit {
                path: path.to_path_buf(),
            },
            severity: Severity::Medium,
            timestamp: now,
        });
    }

    if is_persistence_path(path) {
        return Some(ProcessAlert {
            connection: dummy_connection,
            reason: AlertReason::PersistenceWrite {
                path: path.to_path_buf(),
            },
            severity: Severity::High,
            timestamp: now,
        });
    }

    None
}

pub fn is_lockfile_edit(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|name| {
            matches!(
                name,
                "package-lock.json"
                    | "yarn.lock"
                    | "pnpm-lock.yaml"
                    | "Cargo.lock"
                    | "go.sum"
                    | "Gemfile.lock"
                    | "requirements.txt"
                    | "poetry.lock"
                    | "bun.lockb"
            )
        })
}

pub fn is_persistence_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();

    let patterns = [
        "LaunchAgents/",
        "systemd/",
        "cron.d/",
        "autostart/",
        ".bashrc",
        ".zshrc",
        ".profile",
        ".bash_profile",
        ".vscode/tasks.json",
        ".vscode/settings.json",
        ".claude/settings.json",
    ];

    for pattern in &patterns {
        if path_str.contains(pattern) {
            return true;
        }
    }

    false
}

pub fn is_mcp_config(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    path_str.ends_with(".mcp.json")
        || path_str.ends_with(".cursor/mcp.json")
        || path_str.ends_with(".claude/settings.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persistence_watch_paths_not_empty() {
        let paths = persistence_watch_paths();
        assert!(!paths.is_empty());
    }

    #[test]
    fn test_persistence_watch_paths_includes_claude_dir() {
        let paths = persistence_watch_paths();
        assert!(
            paths
                .iter()
                .any(|p| p.to_string_lossy().contains(".claude"))
        );
    }

    #[test]
    fn test_persistence_watch_paths_includes_vscode_dir() {
        let paths = persistence_watch_paths();
        assert!(
            paths
                .iter()
                .any(|p| p.to_string_lossy().contains(".vscode"))
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_persistence_watch_paths_includes_launch_agents() {
        let paths = persistence_watch_paths();
        assert!(
            paths
                .iter()
                .any(|p| p.to_string_lossy().contains("LaunchAgents"))
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_persistence_watch_paths_includes_systemd() {
        let paths = persistence_watch_paths();
        assert!(
            paths
                .iter()
                .any(|p| p.to_string_lossy().contains("systemd"))
        );
    }

    #[test]
    fn test_evaluate_fs_event_persistence_write() {
        let path = Path::new("/home/user/Library/LaunchAgents/com.evil.plist");
        let alert = evaluate_fs_event(path, FsEventKind::Created);
        assert!(alert.is_some());
        let alert = alert.unwrap();
        assert!(matches!(
            &alert.reason,
            AlertReason::PersistenceWrite { .. }
        ));
        assert_eq!(alert.severity, Severity::High);
    }

    #[test]
    fn test_evaluate_fs_event_lockfile_edit() {
        let path = Path::new("/home/user/project/package-lock.json");
        let alert = evaluate_fs_event(path, FsEventKind::Modified);
        assert!(alert.is_some());
        let alert = alert.unwrap();
        assert!(matches!(&alert.reason, AlertReason::LockfileEdit { .. }));
        assert_eq!(alert.severity, Severity::Medium);
    }

    #[test]
    fn test_evaluate_fs_event_mcp_config_change() {
        let path = Path::new("/home/user/project/.mcp.json");
        let alert = evaluate_fs_event(path, FsEventKind::Modified);
        assert!(alert.is_some());
        let alert = alert.unwrap();
        assert!(matches!(&alert.reason, AlertReason::McpConfigChange { .. }));
        assert_eq!(alert.severity, Severity::High);
    }

    #[test]
    fn test_evaluate_fs_event_cursor_mcp() {
        let path = Path::new("/home/user/.cursor/mcp.json");
        let alert = evaluate_fs_event(path, FsEventKind::Created);
        assert!(alert.is_some());
        let alert = alert.unwrap();
        assert!(matches!(&alert.reason, AlertReason::McpConfigChange { .. }));
    }

    #[test]
    fn test_evaluate_fs_event_claude_settings() {
        let path = Path::new("/home/user/.claude/settings.json");
        let alert = evaluate_fs_event(path, FsEventKind::Modified);
        assert!(alert.is_some());
        let alert = alert.unwrap();
        assert!(matches!(&alert.reason, AlertReason::McpConfigChange { .. }));
    }

    #[test]
    fn test_evaluate_fs_event_unrelated_path() {
        let path = Path::new("/home/user/project/src/main.rs");
        let alert = evaluate_fs_event(path, FsEventKind::Modified);
        assert!(alert.is_none());
    }

    #[test]
    fn test_evaluate_fs_event_deleted_ignored() {
        let path = Path::new("/home/user/project/.mcp.json");
        let alert = evaluate_fs_event(path, FsEventKind::Deleted);
        assert!(alert.is_none());
    }

    #[test]
    fn test_is_lockfile_edit_true() {
        assert!(is_lockfile_edit(Path::new("/project/package-lock.json")));
        assert!(is_lockfile_edit(Path::new("/project/yarn.lock")));
        assert!(is_lockfile_edit(Path::new("/project/pnpm-lock.yaml")));
        assert!(is_lockfile_edit(Path::new("/project/Cargo.lock")));
        assert!(is_lockfile_edit(Path::new("/project/go.sum")));
        assert!(is_lockfile_edit(Path::new("/project/Gemfile.lock")));
        assert!(is_lockfile_edit(Path::new("/project/requirements.txt")));
        assert!(is_lockfile_edit(Path::new("/project/poetry.lock")));
        assert!(is_lockfile_edit(Path::new("/project/bun.lockb")));
    }

    #[test]
    fn test_is_lockfile_edit_false() {
        assert!(!is_lockfile_edit(Path::new("/project/package.json")));
        assert!(!is_lockfile_edit(Path::new("/project/Cargo.toml")));
        assert!(!is_lockfile_edit(Path::new("/project/src/main.rs")));
    }

    #[test]
    fn test_is_persistence_path_true() {
        assert!(is_persistence_path(Path::new("/home/user/.bashrc")));
        assert!(is_persistence_path(Path::new("/home/user/.zshrc")));
        assert!(is_persistence_path(Path::new(
            "/home/user/Library/LaunchAgents/evil.plist"
        )));
        assert!(is_persistence_path(Path::new(
            "/home/user/.config/systemd/user/evil.service"
        )));
        assert!(is_persistence_path(Path::new(
            "/home/user/.vscode/tasks.json"
        )));
        assert!(is_persistence_path(Path::new(
            "/home/user/.claude/settings.json"
        )));
    }

    #[test]
    fn test_is_persistence_path_false() {
        assert!(!is_persistence_path(Path::new(
            "/home/user/project/main.rs"
        )));
        assert!(!is_persistence_path(Path::new(
            "/home/user/Documents/file.txt"
        )));
    }

    #[test]
    fn test_is_mcp_config_true() {
        assert!(is_mcp_config(Path::new("/project/.mcp.json")));
        assert!(is_mcp_config(Path::new("/home/user/.cursor/mcp.json")));
        assert!(is_mcp_config(Path::new("/home/user/.claude/settings.json")));
    }

    #[test]
    fn test_is_mcp_config_false() {
        assert!(!is_mcp_config(Path::new("/project/mcp.json")));
        assert!(!is_mcp_config(Path::new("/project/settings.json")));
        assert!(!is_mcp_config(Path::new("/project/.vscode/settings.json")));
    }
}
