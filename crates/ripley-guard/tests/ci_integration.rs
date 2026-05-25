use std::process::Command;

fn ripley_with_home(home: &std::path::Path) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ripley"));
    cmd.env("HOME", home);
    cmd.env("RIPLEY_DATA_DIR", home.join("data"));
    cmd.env("RIPLEY_CONFIG_DIR", home.join("config"));
    cmd.env("RIPLEY_CACHE_DIR", home.join("cache"));
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
fn test_ci_flag_produces_sarif_file() {
    let tmpdir = tempfile::tempdir().expect("create tempdir");
    let sarif_path = tmpdir.path().join("test-output.sarif");

    let output = ripley_with_home(tmpdir.path())
        .args([
            "scan",
            "--ci",
            "--sarif-output",
            sarif_path.to_str().unwrap(),
        ])
        .arg(fixtures_dir())
        .output()
        .expect("run command");

    assert!(
        output.status.success() || output.status.code() == Some(1),
        "unexpected exit code: {:?}\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        sarif_path.exists(),
        "SARIF file should be written at {sarif_path:?}"
    );

    let content = std::fs::read_to_string(&sarif_path).expect("read sarif");
    let parsed: serde_json::Value = serde_json::from_str(&content).expect("parse JSON");
    assert_eq!(parsed["version"], "2.1.0");
    assert!(parsed["$schema"].as_str().unwrap().contains("sarif"));
}

#[test]
fn test_ci_exit_code_clean() {
    let tmpdir = tempfile::tempdir().expect("create tempdir");
    let fixture_dir = tmpdir.path().join("project");
    std::fs::create_dir_all(&fixture_dir).expect("mkdir");
    std::fs::write(
        fixture_dir.join("package-lock.json"),
        r#"{
  "name": "clean-project",
  "version": "1.0.0",
  "lockfileVersion": 3,
  "requires": true,
  "packages": {}
}"#,
    )
    .expect("write fixture");

    let output = ripley_with_home(tmpdir.path())
        .args(["scan", "--ci"])
        .arg(&fixture_dir)
        .output()
        .expect("run command");

    assert_eq!(
        output.status.code(),
        Some(0),
        "clean project should exit 0\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_sarif_output_file_alongside_table() {
    let tmpdir = tempfile::tempdir().expect("create tempdir");
    let sarif_path = tmpdir.path().join("side.sarif");

    let output = ripley_with_home(tmpdir.path())
        .args([
            "scan",
            "--ci",
            "--format",
            "table",
            "--sarif-output",
            sarif_path.to_str().unwrap(),
        ])
        .arg(fixtures_dir())
        .output()
        .expect("run command");

    assert!(
        output.status.success() || output.status.code() == Some(1),
        "unexpected exit: {:?}\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("$schema"),
        "stdout should be table format, not SARIF"
    );

    assert!(sarif_path.exists(), "SARIF sidecar should be written");
    let content = std::fs::read_to_string(&sarif_path).expect("read sarif");
    let parsed: serde_json::Value = serde_json::from_str(&content).expect("parse JSON");
    assert_eq!(parsed["version"], "2.1.0");
}

#[test]
fn test_sarif_format_does_not_write_sidecar() {
    let tmpdir = tempfile::tempdir().expect("create tempdir");
    let sarif_path = tmpdir.path().join("no-write.sarif");

    let output = ripley_with_home(tmpdir.path())
        .args([
            "scan",
            "--ci",
            "--format",
            "sarif",
            "--sarif-output",
            sarif_path.to_str().unwrap(),
        ])
        .arg(fixtures_dir())
        .output()
        .expect("run command");

    assert!(
        output.status.success() || output.status.code() == Some(1),
        "unexpected exit: {:?}\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("$schema"),
        "stdout should contain SARIF when --format sarif"
    );

    assert!(
        !sarif_path.exists(),
        "sidecar should not be written when format is already sarif"
    );
}
