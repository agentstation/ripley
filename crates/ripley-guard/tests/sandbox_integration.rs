use std::path::PathBuf;

use ripley_core::sandbox::{SandboxProfile, ViolationKind, execute_sandboxed};

fn fixtures_scripts_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("fixtures")
        .join("scripts")
}

#[test]
fn test_sandbox_disabled_passthrough() {
    let config = ripley_core::config::GuardConfig::default();
    assert!(!config.sandbox);
}

#[test]
fn test_sandbox_config_toml_roundtrip() {
    let tmpdir = tempfile::tempdir().expect("create tmpdir");
    let config_dir = tmpdir.path().join(".config").join("ripley");
    std::fs::create_dir_all(&config_dir).expect("create config dir");
    std::fs::write(
        config_dir.join("config.toml"),
        "[guard]\nsandbox = true\nsandbox_allow_network = false\n",
    )
    .expect("write config");

    let project_config = tmpdir.path().join(".ripley.toml");
    std::fs::write(&project_config, "[guard]\nsandbox = true\n").expect("write project config");

    let config = ripley_core::config::load_config(tmpdir.path()).expect("load config");
    assert!(config.guard.sandbox);
    assert!(!config.guard.sandbox_allow_network);
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;

    #[test]
    #[ignore] // sandbox-exec behavior varies by macOS version
    fn test_sandbox_blocks_network() {
        let tmpdir = tempfile::tempdir().expect("create tempdir");
        let profile = SandboxProfile {
            allow_network: false,
            writable_paths: vec![tmpdir.path().to_path_buf()],
            readable_paths: vec![
                PathBuf::from("/usr"),
                PathBuf::from("/bin"),
                PathBuf::from("/lib"),
                PathBuf::from("/dev/null"),
                PathBuf::from("/dev/urandom"),
            ],
            working_dir: tmpdir.path().to_path_buf(),
        };

        let script =
            std::fs::read_to_string(fixtures_scripts_dir().join("sandbox-network-test.sh"))
                .expect("read fixture");

        let result = execute_sandboxed(&script, &[], &profile);
        match result {
            Ok(r) => {
                assert!(
                    r.network_blocked || r.exit_code != 0,
                    "expected network to be blocked or non-zero exit"
                );
            }
            Err(ripley_core::sandbox::SandboxError::SandboxExecNotFound) => {}
            Err(e) => panic!("unexpected error: {e}"),
        }
    }

    #[test]
    #[ignore] // sandbox-exec behavior varies by macOS version
    fn test_sandbox_allows_benign() {
        let tmpdir = tempfile::tempdir().expect("create tempdir");
        let profile = SandboxProfile {
            allow_network: false,
            writable_paths: vec![tmpdir.path().to_path_buf()],
            readable_paths: vec![
                PathBuf::from("/usr"),
                PathBuf::from("/bin"),
                PathBuf::from("/lib"),
                PathBuf::from("/dev/null"),
                PathBuf::from("/dev/urandom"),
            ],
            working_dir: tmpdir.path().to_path_buf(),
        };

        let script = std::fs::read_to_string(fixtures_scripts_dir().join("sandbox-benign.sh"))
            .expect("read fixture");

        let result = execute_sandboxed(&script, &[], &profile);
        match result {
            Ok(r) => {
                assert_eq!(r.exit_code, 0, "benign script should succeed");
                assert!(
                    r.stdout_output.contains("benign-complete"),
                    "expected benign output"
                );
            }
            Err(ripley_core::sandbox::SandboxError::SandboxExecNotFound) => {}
            Err(e) => panic!("unexpected error: {e}"),
        }
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use super::*;

    fn has_bwrap() -> bool {
        // Verify bwrap can actually open a minimal sandbox — `--version`
        // succeeds even when user namespaces are disabled (e.g. some CI
        // runners), which would make every sandboxed exec fail.
        std::process::Command::new("bwrap")
            .args([
                "--ro-bind",
                "/",
                "/",
                "--dev",
                "/dev",
                "--proc",
                "/proc",
                "--unshare-all",
                "true",
            ])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    #[test]
    fn test_sandbox_blocks_network() {
        if !has_bwrap() {
            eprintln!("skipping: bwrap not installed");
            return;
        }

        let tmpdir = tempfile::tempdir().expect("create tmpdir");
        let profile = SandboxProfile {
            allow_network: false,
            writable_paths: vec![tmpdir.path().to_path_buf()],
            readable_paths: vec![
                PathBuf::from("/usr"),
                PathBuf::from("/bin"),
                PathBuf::from("/lib"),
            ],
            working_dir: tmpdir.path().to_path_buf(),
        };

        let script =
            std::fs::read_to_string(fixtures_scripts_dir().join("sandbox-network-test.sh"))
                .expect("read fixture");

        let result = execute_sandboxed(&script, &[], &profile).expect("execute");
        assert!(
            result.network_blocked || result.exit_code != 0,
            "expected network to be blocked or non-zero exit"
        );
    }

    #[test]
    fn test_sandbox_allows_benign() {
        if !has_bwrap() {
            eprintln!("skipping: bwrap not installed");
            return;
        }

        let tmpdir = tempfile::tempdir().expect("create tmpdir");
        let profile = SandboxProfile {
            allow_network: false,
            writable_paths: vec![tmpdir.path().to_path_buf()],
            readable_paths: vec![
                PathBuf::from("/usr"),
                PathBuf::from("/bin"),
                PathBuf::from("/lib"),
            ],
            working_dir: tmpdir.path().to_path_buf(),
        };

        let script = std::fs::read_to_string(fixtures_scripts_dir().join("sandbox-benign.sh"))
            .expect("read fixture");

        let result = execute_sandboxed(&script, &[], &profile).expect("execute");
        assert_eq!(result.exit_code, 0, "benign script should succeed");
        assert!(
            result.stdout_output.contains("benign-complete"),
            "expected benign output"
        );
    }
}

#[test]
fn test_sandbox_result_logged_format() {
    let result = ripley_core::sandbox::SandboxResult {
        exit_code: 1,
        stderr_output: "sandbox violation".to_string(),
        stdout_output: String::new(),
        network_blocked: true,
        duration_ms: 100,
        sandbox_violations: vec![ripley_core::sandbox::SandboxViolation {
            kind: ViolationKind::NetworkAccess,
            detail: "denied connect to 93.184.216.34".to_string(),
        }],
    };

    let json = serde_json::to_string(&result).expect("serialize");
    assert!(json.contains("\"network_blocked\":true"));
    assert!(json.contains("\"network_access\""));
    assert!(json.contains("denied connect"));
}

#[test]
fn test_sandbox_profile_for_ecosystem() {
    let profile =
        SandboxProfile::for_ecosystem(ripley_core::types::Ecosystem::Npm, &PathBuf::from("/pkg"));
    assert!(!profile.allow_network);
    assert_eq!(profile.working_dir, PathBuf::from("/pkg"));
    assert!(profile.writable_paths.contains(&PathBuf::from("/pkg")));
}

#[test]
fn test_sandbox_profile_respects_config_overrides() {
    let pkg = PathBuf::from("/test/pkg");
    let mut profile = SandboxProfile::for_ecosystem(ripley_core::types::Ecosystem::Npm, &pkg);
    profile.allow_network = true;
    profile.writable_paths.push(PathBuf::from("/extra/path"));

    assert!(profile.allow_network);
    assert!(
        profile
            .writable_paths
            .contains(&PathBuf::from("/extra/path"))
    );
}
