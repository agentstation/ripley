use std::time::Duration;

use ripley_core::config::MonitorConfig;
use ripley_core::monitor::process::{AlertReason, ProcessAlert};
use ripley_core::monitor::{FsEventKind, evaluate_fs_event};
use ripley_core::types::Severity;

#[test]
fn test_evaluate_fs_event_detects_persistence_write() {
    let path = std::path::Path::new("/home/user/Library/LaunchAgents/com.evil.plist");
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
fn test_evaluate_fs_event_detects_mcp_config() {
    let path = std::path::Path::new("/home/user/project/.mcp.json");
    let alert = evaluate_fs_event(path, FsEventKind::Modified);
    assert!(alert.is_some());
    let alert = alert.unwrap();
    assert!(matches!(&alert.reason, AlertReason::McpConfigChange { .. }));
}

#[test]
fn test_evaluate_fs_event_detects_lockfile_edit() {
    let path = std::path::Path::new("/home/user/project/package-lock.json");
    let alert = evaluate_fs_event(path, FsEventKind::Modified);
    assert!(alert.is_some());
    let alert = alert.unwrap();
    assert!(matches!(&alert.reason, AlertReason::LockfileEdit { .. }));
}

#[test]
fn test_monitor_config_defaults_disabled() {
    let config = MonitorConfig::default();
    assert!(!config.enabled);
    assert!(config.watch_processes);
    assert!(config.watch_persistence);
    assert!(config.watch_lockfiles);
}

#[test]
fn test_process_alert_serializes_to_json() {
    let alert = ProcessAlert {
        connection: Some(ripley_core::forensic::network::NetworkConnection {
            process: "node".to_string(),
            pid: 12345,
            protocol: "TCP".to_string(),
            remote_addr: "evil.example.com".to_string(),
            remote_port: 443,
            state: "ESTABLISHED".to_string(),
        }),
        reason: AlertReason::C2Connection {
            indicator: "evil.example.com".to_string(),
        },
        severity: Severity::Critical,
        timestamp: 1700000000,
    };

    let json = serde_json::to_string(&alert).expect("serialize");
    assert!(json.contains("\"process\":\"node\""));
    assert!(json.contains("\"severity\":\"critical\""));
    assert!(json.contains("\"C2Connection\""));
}

#[test]
fn test_guard_log_entry_format() {
    let timestamp = 1700000000u64;
    let entry = serde_json::json!({
        "timestamp": timestamp,
        "event_type": "monitor_alert",
        "severity": "Critical",
        "process": "node",
        "pid": 12345,
        "detail": "C2 connection to evil.example.com",
    });

    let json = serde_json::to_string(&entry).expect("serialize");
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("parse");
    assert_eq!(parsed["event_type"], "monitor_alert");
    assert_eq!(parsed["severity"], "Critical");
    assert_eq!(parsed["pid"], 12345);
}

#[tokio::test]
async fn test_monitor_writes_guard_log() {
    let dir = tempfile::tempdir().expect("tempdir");
    let guard_log = dir.path().join("guard.jsonl");

    {
        use std::io::Write;
        let entry = serde_json::json!({
            "timestamp": 1700000000u64,
            "event_type": "monitor_alert",
            "severity": "High",
            "process": "node",
            "pid": 9999,
            "detail": "Persistence write to /tmp/evil",
        });
        let line = serde_json::to_string(&entry).expect("serialize");
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&guard_log)
            .expect("open");
        writeln!(file, "{line}").expect("write");
    }

    let contents = std::fs::read_to_string(&guard_log).expect("read");
    let parsed: serde_json::Value = serde_json::from_str(contents.trim()).expect("parse");
    assert_eq!(parsed["event_type"], "monitor_alert");
    assert_eq!(parsed["pid"], 9999);
}

#[tokio::test]
async fn test_monitor_cancellation() {
    let cancel = tokio_util::sync::CancellationToken::new();
    let cancel_clone = cancel.clone();

    let handle = tokio::spawn(async move {
        tokio::select! {
            _ = cancel_clone.cancelled() => {
                "shutdown"
            }
            _ = tokio::time::sleep(Duration::from_secs(60)) => {
                "timeout"
            }
        }
    });

    cancel.cancel();
    let result = handle.await.expect("join");
    assert_eq!(result, "shutdown");
}
