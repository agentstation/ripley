use std::path::{Path, PathBuf};

use crate::types::Ecosystem;

#[derive(Debug, Clone)]
pub struct SandboxProfile {
    pub allow_network: bool,
    pub writable_paths: Vec<PathBuf>,
    pub readable_paths: Vec<PathBuf>,
    pub working_dir: PathBuf,
}

impl SandboxProfile {
    pub fn for_ecosystem(ecosystem: Ecosystem, package_dir: &Path) -> Self {
        match ecosystem {
            Ecosystem::Npm => Self::default_for_npm(package_dir),
            Ecosystem::Cargo => Self::default_for_cargo(package_dir),
            Ecosystem::PyPI => Self::default_for_pip(package_dir),
            _ => Self::default_for_npm(package_dir),
        }
    }

    pub fn default_for_npm(package_dir: &Path) -> Self {
        Self {
            allow_network: false,
            writable_paths: vec![
                package_dir.to_path_buf(),
                package_dir.join("node_modules/.cache"),
            ],
            readable_paths: vec![
                PathBuf::from("/usr"),
                PathBuf::from("/bin"),
                PathBuf::from("/lib"),
                PathBuf::from("/dev/null"),
                PathBuf::from("/dev/urandom"),
            ],
            working_dir: package_dir.to_path_buf(),
        }
    }

    pub fn default_for_pip(package_dir: &Path) -> Self {
        Self {
            allow_network: false,
            writable_paths: vec![package_dir.to_path_buf()],
            readable_paths: vec![
                PathBuf::from("/usr"),
                PathBuf::from("/bin"),
                PathBuf::from("/lib"),
                PathBuf::from("/dev/null"),
                PathBuf::from("/dev/urandom"),
            ],
            working_dir: package_dir.to_path_buf(),
        }
    }

    pub fn default_for_cargo(package_dir: &Path) -> Self {
        Self {
            allow_network: false,
            writable_paths: vec![package_dir.to_path_buf()],
            readable_paths: vec![
                PathBuf::from("/usr"),
                PathBuf::from("/bin"),
                PathBuf::from("/lib"),
                PathBuf::from("/dev/null"),
                PathBuf::from("/dev/urandom"),
            ],
            working_dir: package_dir.to_path_buf(),
        }
    }

    #[cfg(target_os = "macos")]
    pub fn to_sandbox_exec_profile(&self) -> String {
        let mut profile = String::new();
        profile.push_str("(version 1)\n");
        profile.push_str("(deny default)\n");
        profile.push_str("(allow process-exec)\n");
        profile.push_str("(allow process-fork)\n");
        profile.push_str("(allow sysctl-read)\n");
        profile.push_str("(allow mach-lookup)\n");

        for path in &self.readable_paths {
            profile.push_str(&format!(
                "(allow file-read* (subpath \"{}\"))\n",
                path.display()
            ));
        }

        for path in &self.writable_paths {
            profile.push_str(&format!(
                "(allow file-read* (subpath \"{}\"))\n",
                path.display()
            ));
            profile.push_str(&format!(
                "(allow file-write* (subpath \"{}\"))\n",
                path.display()
            ));
        }

        if self.allow_network {
            profile.push_str("(allow network*)\n");
        } else {
            profile.push_str("(deny network*)\n");
        }

        profile
    }

    #[cfg(target_os = "linux")]
    pub fn to_bwrap_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        args.extend(["--ro-bind".to_string(), "/".to_string(), "/".to_string()]);
        args.extend(["--dev".to_string(), "/dev".to_string()]);
        args.extend(["--proc".to_string(), "/proc".to_string()]);

        for path in &self.writable_paths {
            let p = path.to_string_lossy().to_string();
            args.extend(["--bind".to_string(), p.clone(), p]);
        }

        if !self.allow_network {
            args.push("--unshare-net".to_string());
        }

        args.extend([
            "--chdir".to_string(),
            self.working_dir.to_string_lossy().to_string(),
        ]);

        args
    }

    #[cfg(target_os = "windows")]
    pub fn to_sandbox_exec_profile(&self) -> String {
        String::new()
    }

    #[cfg(target_os = "windows")]
    pub fn to_bwrap_args(&self) -> Vec<String> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_for_npm_no_network() {
        let profile = SandboxProfile::default_for_npm(Path::new("/project/node_modules/pkg"));
        assert!(!profile.allow_network);
        assert!(
            profile
                .writable_paths
                .contains(&PathBuf::from("/project/node_modules/pkg"))
        );
        assert!(
            !profile
                .writable_paths
                .iter()
                .any(|p| p.starts_with("/home"))
        );
    }

    #[test]
    fn test_for_ecosystem_dispatch() {
        let profile = SandboxProfile::for_ecosystem(Ecosystem::Npm, Path::new("/pkg"));
        assert!(!profile.allow_network);
        assert_eq!(profile.working_dir, PathBuf::from("/pkg"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_macos_profile_denies_network() {
        let profile = SandboxProfile {
            allow_network: false,
            writable_paths: vec![PathBuf::from("/tmp/pkg")],
            readable_paths: vec![PathBuf::from("/usr")],
            working_dir: PathBuf::from("/tmp/pkg"),
        };
        let sb = profile.to_sandbox_exec_profile();
        assert!(sb.contains("(deny default)"));
        assert!(sb.contains("(deny network*)"));
        assert!(!sb.contains("(allow network*)"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_macos_profile_allows_network_when_enabled() {
        let profile = SandboxProfile {
            allow_network: true,
            writable_paths: vec![PathBuf::from("/tmp/pkg")],
            readable_paths: vec![PathBuf::from("/usr")],
            working_dir: PathBuf::from("/tmp/pkg"),
        };
        let sb = profile.to_sandbox_exec_profile();
        assert!(sb.contains("(allow network*)"));
        assert!(!sb.contains("(deny network*)"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_linux_bwrap_unshare_net() {
        let profile = SandboxProfile {
            allow_network: false,
            writable_paths: vec![PathBuf::from("/tmp/pkg")],
            readable_paths: vec![PathBuf::from("/usr")],
            working_dir: PathBuf::from("/tmp/pkg"),
        };
        let args = profile.to_bwrap_args();
        assert!(args.contains(&"--unshare-net".to_string()));
        assert!(args.contains(&"--ro-bind".to_string()));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_linux_bwrap_bind_writable() {
        let profile = SandboxProfile {
            allow_network: false,
            writable_paths: vec![PathBuf::from("/tmp/pkg")],
            readable_paths: vec![PathBuf::from("/usr")],
            working_dir: PathBuf::from("/tmp/pkg"),
        };
        let args = profile.to_bwrap_args();
        assert!(args.contains(&"--bind".to_string()));
        assert!(args.contains(&"/tmp/pkg".to_string()));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_linux_bwrap_no_unshare_when_network_allowed() {
        let profile = SandboxProfile {
            allow_network: true,
            writable_paths: Vec::new(),
            readable_paths: Vec::new(),
            working_dir: PathBuf::from("/tmp"),
        };
        let args = profile.to_bwrap_args();
        assert!(!args.contains(&"--unshare-net".to_string()));
    }
}
