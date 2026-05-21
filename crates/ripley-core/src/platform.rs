use std::path::{Path, PathBuf};

pub fn home_dir() -> Option<PathBuf> {
    #[cfg(unix)]
    {
        std::env::var("HOME").ok().map(PathBuf::from)
    }
    #[cfg(windows)]
    {
        std::env::var("USERPROFILE").ok().map(PathBuf::from)
    }
}

pub fn shell_rc_files() -> &'static [&'static str] {
    #[cfg(target_os = "macos")]
    {
        &[
            ".zshrc",
            ".bashrc",
            ".profile",
            ".bash_profile",
            ".zprofile",
        ]
    }
    #[cfg(target_os = "linux")]
    {
        &[
            ".bashrc",
            ".profile",
            ".bash_profile",
            ".zshrc",
            ".zprofile",
        ]
    }
    #[cfg(target_os = "windows")]
    {
        &[]
    }
}

pub fn persistence_dirs(home: &Path) -> Vec<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        vec![home.join("Library/LaunchAgents")]
    }
    #[cfg(target_os = "linux")]
    {
        vec![
            home.join(".config/systemd/user"),
            home.join(".config/autostart"),
        ]
    }
    #[cfg(target_os = "windows")]
    {
        let mut dirs = Vec::new();
        if let Some(appdata) = std::env::var("APPDATA").ok() {
            dirs.push(
                PathBuf::from(appdata).join("Microsoft\\Windows\\Start Menu\\Programs\\Startup"),
            );
        }
        dirs
    }
}

pub fn detect_shell_rc(home: &Path) -> Option<PathBuf> {
    #[cfg(unix)]
    {
        let shell = std::env::var("SHELL").unwrap_or_default();

        if shell.ends_with("zsh") {
            return Some(home.join(".zshrc"));
        }
        if shell.ends_with("bash") {
            let bashrc = home.join(".bashrc");
            if bashrc.exists() {
                return Some(bashrc);
            }
            return Some(home.join(".profile"));
        }
        for rc in shell_rc_files() {
            let path = home.join(rc);
            if path.exists() {
                return Some(path);
            }
        }
        Some(home.join(".profile"))
    }
    #[cfg(windows)]
    {
        let _ = home;
        if let Some(docs) = std::env::var("USERPROFILE").ok() {
            let ps_profile = PathBuf::from(docs)
                .join("Documents\\WindowsPowerShell\\Microsoft.PowerShell_profile.ps1");
            if ps_profile.exists() {
                return Some(ps_profile);
            }
        }
        None
    }
}

pub fn path_var_separator() -> char {
    #[cfg(unix)]
    {
        ':'
    }
    #[cfg(windows)]
    {
        ';'
    }
}

pub fn expand_tilde(pattern: &str) -> String {
    if pattern.starts_with("~/")
        && let Some(home) = home_dir()
    {
        return format!("{}{}", home.display(), &pattern[1..]);
    }
    pattern.to_string()
}

pub fn launch_terminal(
    command: &str,
    args: &[&str],
    working_dir: &Path,
) -> Result<std::process::Child, std::io::Error> {
    #[cfg(target_os = "macos")]
    {
        let mut all_args = vec![command];
        all_args.extend_from_slice(args);
        let script = all_args.join(" ");
        std::process::Command::new("open")
            .arg("-a")
            .arg("Terminal.app")
            .arg("--args")
            .arg("bash")
            .arg("-c")
            .arg(&script)
            .current_dir(working_dir)
            .spawn()
    }
    #[cfg(target_os = "linux")]
    {
        let mut all_args = vec![command];
        all_args.extend_from_slice(args);
        let script = all_args.join(" ");
        let terminals = ["x-terminal-emulator", "gnome-terminal", "xterm"];
        for term in &terminals {
            if which_exists(term) {
                return std::process::Command::new(term)
                    .arg("-e")
                    .arg(&script)
                    .current_dir(working_dir)
                    .spawn();
            }
        }
        std::process::Command::new("xterm")
            .arg("-e")
            .arg(&script)
            .current_dir(working_dir)
            .spawn()
    }
    #[cfg(target_os = "windows")]
    {
        let mut all_args = vec![command];
        all_args.extend_from_slice(args);
        let script = all_args.join(" ");
        std::process::Command::new("cmd")
            .arg("/C")
            .arg("start")
            .arg("cmd")
            .arg("/K")
            .arg(&script)
            .current_dir(working_dir)
            .spawn()
    }
}

pub fn is_command_available(cmd: &str) -> bool {
    which_exists(cmd)
}

fn which_exists(cmd: &str) -> bool {
    #[cfg(unix)]
    {
        std::process::Command::new("which")
            .arg(cmd)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    }
    #[cfg(windows)]
    {
        std::process::Command::new("where")
            .arg(cmd)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_home_dir_returns_some() {
        let home = home_dir();
        assert!(home.is_some());
        assert!(home.as_ref().is_some_and(|p| p.is_absolute()));
    }

    #[test]
    fn test_shell_rc_files_not_empty_on_unix() {
        #[cfg(unix)]
        assert!(!shell_rc_files().is_empty());
    }

    #[test]
    fn test_persistence_dirs_returns_paths() {
        if let Some(home) = home_dir() {
            let dirs = persistence_dirs(&home);
            assert!(!dirs.is_empty());
            for d in &dirs {
                assert!(d.is_absolute());
            }
        }
    }

    #[test]
    fn test_detect_shell_rc_returns_path() {
        if let Some(home) = home_dir() {
            let rc = detect_shell_rc(&home);
            assert!(rc.is_some());
            assert!(rc.as_ref().is_some_and(|p| p.is_absolute()));
        }
    }

    #[test]
    fn test_path_var_separator() {
        #[cfg(unix)]
        assert_eq!(path_var_separator(), ':');
        #[cfg(windows)]
        assert_eq!(path_var_separator(), ';');
    }

    #[test]
    fn test_expand_tilde() {
        if home_dir().is_some() {
            let expanded = expand_tilde("~/test/path");
            assert!(!expanded.starts_with("~/"));
            assert!(expanded.ends_with("/test/path"));
        }
        assert_eq!(expand_tilde("/absolute/path"), "/absolute/path");
        assert_eq!(expand_tilde("relative/path"), "relative/path");
    }

    #[test]
    fn test_is_command_available() {
        #[cfg(unix)]
        {
            assert!(is_command_available("ls"));
            assert!(!is_command_available("nonexistent_binary_xyz_123"));
        }
    }
}
