//! Capability ACL boundary tests.

use std::path::PathBuf;

fn capability_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("capabilities")
}

#[test]
fn default_capability_parses_as_json() {
    let path = capability_dir().join("default.json");
    let body = std::fs::read_to_string(&path).expect("read default.json");
    let value: serde_json::Value = serde_json::from_str(&body).expect("parse default.json");
    assert_eq!(value["identifier"], "default");
    assert_eq!(value["windows"], serde_json::json!(["main"]));
}

#[test]
fn default_capability_grants_minimum_required_permissions() {
    let path = capability_dir().join("default.json");
    let body = std::fs::read_to_string(&path).expect("read default.json");
    let value: serde_json::Value = serde_json::from_str(&body).expect("parse default.json");
    let perms = value["permissions"]
        .as_array()
        .expect("permissions array")
        .iter()
        .filter_map(|p| p.as_str())
        .collect::<Vec<_>>();

    for required in [
        "core:default",
        "core:tray:default",
        "global-shortcut:default",
    ] {
        assert!(
            perms.contains(&required),
            "default.json missing required permission `{required}` (got {perms:?})"
        );
    }
}

#[test]
fn default_capability_does_not_grant_dangerous_fs_or_shell_perms() {
    let path = capability_dir().join("default.json");
    let body = std::fs::read_to_string(&path).expect("read default.json");
    let value: serde_json::Value = serde_json::from_str(&body).expect("parse default.json");
    let perms = value["permissions"]
        .as_array()
        .expect("permissions array")
        .iter()
        .filter_map(|p| p.as_str())
        .collect::<Vec<_>>();

    for forbidden in [
        "fs:allow-write-text-file",
        "fs:allow-remove",
        "shell:allow-execute",
        "shell:allow-spawn",
        "http:default",
    ] {
        assert!(
            !perms.contains(&forbidden),
            "default.json grants `{forbidden}` which is outside the M26 boundary"
        );
    }
}
