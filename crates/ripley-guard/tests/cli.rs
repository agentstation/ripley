use std::process::Command;

fn ripley() -> Command {
    let bin = env!("CARGO_BIN_EXE_ripley");
    Command::new(bin)
}

fn ripley_with_temp_home(home: &std::path::Path) -> Command {
    let mut cmd = ripley();
    cmd.env("HOME", home);
    cmd
}

fn fixtures_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("fixtures")
}

#[test]
fn test_help_exits_zero() {
    let output = ripley().arg("--help").output().expect("run ripley");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Supply chain defense"));
}

#[test]
fn test_scan_no_lockfiles_exits_two() {
    let home = tempfile::tempdir().expect("tempdir");
    let scan_dir = tempfile::tempdir().expect("tempdir");
    let output = ripley_with_temp_home(home.path())
        .args(["scan", scan_dir.path().to_str().unwrap()])
        .output()
        .expect("run ripley");
    assert_eq!(
        output.status.code(),
        Some(2),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("No lockfiles found"),
        "unexpected stderr: {stderr}"
    );
}

#[test]
fn test_scan_fixtures_table_format() {
    let home = tempfile::tempdir().expect("tempdir");
    let output = ripley_with_temp_home(home.path())
        .args(["scan", "--no-cache", fixtures_dir().to_str().unwrap()])
        .output()
        .expect("run ripley");
    // Scan may find vulns (exit 1) or not (exit 0), but should never error (exit 2)
    assert!(
        output.status.code() == Some(0) || output.status.code() == Some(1),
        "expected exit 0 or 1, got {:?}\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_scan_fixtures_json_format() {
    let home = tempfile::tempdir().expect("tempdir");
    let output = ripley_with_temp_home(home.path())
        .args([
            "scan",
            "--format",
            "json",
            "--no-cache",
            fixtures_dir().to_str().unwrap(),
        ])
        .output()
        .expect("run ripley");
    assert!(
        output.status.code() == Some(0) || output.status.code() == Some(1),
        "expected exit 0 or 1, got {:?}\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON output");
    assert!(parsed.get("matches").is_some());
    assert!(parsed.get("posture_warnings").is_some());
    assert!(parsed.get("risky_specs").is_some());
}

#[test]
fn test_config_path() {
    let output = ripley()
        .args(["config", "--path"])
        .output()
        .expect("run ripley");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.trim().ends_with("config.toml"));
}

#[test]
fn test_status_runs() {
    let home = tempfile::tempdir().expect("tempdir");
    let output = ripley_with_temp_home(home.path())
        .arg("status")
        .output()
        .expect("run ripley");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Ripley Status"));
    assert!(stdout.contains("Advisory cache:"));
}

#[test]
fn test_unknown_subcommand_exits_nonzero() {
    let output = ripley().arg("nonexistent").output().expect("run ripley");
    assert!(!output.status.success());
}

#[test]
fn test_guard_status_not_installed() {
    let home = tempfile::tempdir().expect("tempdir");
    let output = ripley_with_temp_home(home.path())
        .args(["guard", "status"])
        .output()
        .expect("run ripley");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("not installed"));
    assert!(stdout.contains("trusted packages: 0"));
}

#[test]
fn test_guard_trust_untrust() {
    let home = tempfile::tempdir().expect("tempdir");

    let output = ripley_with_temp_home(home.path())
        .args(["guard", "trust", "express"])
        .output()
        .expect("run ripley");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Trusted:"));

    let output = ripley_with_temp_home(home.path())
        .args(["guard", "trust", "express"])
        .output()
        .expect("run ripley");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("already trusted"));

    let output = ripley_with_temp_home(home.path())
        .args(["guard", "status"])
        .output()
        .expect("run ripley");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("trusted packages: 1"));

    let output = ripley_with_temp_home(home.path())
        .args(["guard", "untrust", "express"])
        .output()
        .expect("run ripley");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Untrusted:"));

    let output = ripley_with_temp_home(home.path())
        .args(["guard", "status"])
        .output()
        .expect("run ripley");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("trusted packages: 0"));
}

#[test]
fn test_guard_log_empty() {
    let home = tempfile::tempdir().expect("tempdir");
    let output = ripley_with_temp_home(home.path())
        .args(["guard", "log"])
        .output()
        .expect("run ripley");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No guard log entries"));
}
