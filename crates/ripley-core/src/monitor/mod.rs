pub mod contain;
pub mod filesystem;
pub mod process;

use std::path::PathBuf;

use serde::Serialize;

use crate::types::Severity;

pub use contain::{ContainResult, ProcessSnapshot, collect_process_snapshot, contain_process};
pub use filesystem::{
    FsEventKind, evaluate_fs_event, is_lockfile_edit, is_mcp_config, is_persistence_path,
    persistence_watch_paths,
};
pub use process::{
    AlertReason, ProcessAlert, evaluate_connections, is_developer_process, merge_c2_database,
    scan_processes,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MonitorEventType {
    ProcessAlert,
    PersistenceAlert,
    LockfileAlert,
    McpConfigAlert,
    ContainAction,
    MonitorStarted,
    MonitorStopped,
}

#[derive(Debug, Clone, Serialize)]
pub struct MonitorLogEntry {
    pub timestamp: u64,
    pub event_type: MonitorEventType,
    pub severity: Option<Severity>,
    pub process: Option<String>,
    pub pid: Option<u32>,
    pub path: Option<PathBuf>,
    pub detail: String,
}

impl MonitorLogEntry {
    pub fn from_alert(alert: &ProcessAlert) -> Self {
        let event_type = match &alert.reason {
            AlertReason::PersistenceWrite { .. } => MonitorEventType::PersistenceAlert,
            AlertReason::LockfileEdit { .. } => MonitorEventType::LockfileAlert,
            AlertReason::McpConfigChange { .. } => MonitorEventType::McpConfigAlert,
            _ => MonitorEventType::ProcessAlert,
        };

        let path = match &alert.reason {
            AlertReason::PersistenceWrite { path }
            | AlertReason::LockfileEdit { path }
            | AlertReason::McpConfigChange { path }
            | AlertReason::UnknownBinary { path } => Some(path.clone()),
            _ => None,
        };

        Self {
            timestamp: alert.timestamp,
            event_type,
            severity: Some(alert.severity),
            process: if alert.connection.pid > 0 {
                Some(alert.connection.process.clone())
            } else {
                None
            },
            pid: if alert.connection.pid > 0 {
                Some(alert.connection.pid)
            } else {
                None
            },
            path,
            detail: format!("{}", alert.reason),
        }
    }

    pub fn lifecycle(event_type: MonitorEventType, detail: &str) -> Self {
        Self {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            event_type,
            severity: None,
            process: None,
            pid: None,
            path: None,
            detail: detail.to_string(),
        }
    }
}

pub fn format_log_entry(entry: &MonitorLogEntry) -> Result<String, serde_json::Error> {
    serde_json::to_string(entry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forensic::network::NetworkConnection;

    #[test]
    fn test_monitor_log_entry_from_alert() {
        let alert = ProcessAlert {
            connection: NetworkConnection {
                process: "node".to_string(),
                pid: 1234,
                protocol: "TCP".to_string(),
                remote_addr: "evil.example.com".to_string(),
                remote_port: 443,
                state: "ESTABLISHED".to_string(),
            },
            reason: AlertReason::C2Connection {
                indicator: "evil.example.com".to_string(),
            },
            severity: Severity::Critical,
            timestamp: 1700000000,
        };

        let entry = MonitorLogEntry::from_alert(&alert);
        assert!(matches!(entry.event_type, MonitorEventType::ProcessAlert));
        assert_eq!(entry.severity, Some(Severity::Critical));
        assert_eq!(entry.process, Some("node".to_string()));
        assert_eq!(entry.pid, Some(1234));
        assert!(entry.detail.contains("evil.example.com"));
    }

    #[test]
    fn test_monitor_log_entry_persistence() {
        let alert = ProcessAlert {
            connection: NetworkConnection {
                process: String::new(),
                pid: 0,
                protocol: String::new(),
                remote_addr: String::new(),
                remote_port: 0,
                state: String::new(),
            },
            reason: AlertReason::PersistenceWrite {
                path: PathBuf::from("/tmp/evil.plist"),
            },
            severity: Severity::High,
            timestamp: 1700000000,
        };

        let entry = MonitorLogEntry::from_alert(&alert);
        assert!(matches!(
            entry.event_type,
            MonitorEventType::PersistenceAlert
        ));
        assert_eq!(entry.path, Some(PathBuf::from("/tmp/evil.plist")));
        assert!(entry.process.is_none());
        assert!(entry.pid.is_none());
    }

    #[test]
    fn test_monitor_log_entry_lifecycle() {
        let entry =
            MonitorLogEntry::lifecycle(MonitorEventType::MonitorStarted, "Monitor daemon started");
        assert!(matches!(entry.event_type, MonitorEventType::MonitorStarted));
        assert!(entry.severity.is_none());
        assert_eq!(entry.detail, "Monitor daemon started");
        assert!(entry.timestamp > 0);
    }

    #[test]
    fn test_format_log_entry_valid_json() {
        let entry =
            MonitorLogEntry::lifecycle(MonitorEventType::MonitorStopped, "Monitor daemon stopped");
        let json = format_log_entry(&entry).expect("serialize");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("parse");
        assert_eq!(parsed["event_type"], "monitor_stopped");
        assert_eq!(parsed["detail"], "Monitor daemon stopped");
    }

    #[test]
    fn test_all_event_types_serialize() {
        let types = [
            MonitorEventType::ProcessAlert,
            MonitorEventType::PersistenceAlert,
            MonitorEventType::LockfileAlert,
            MonitorEventType::McpConfigAlert,
            MonitorEventType::ContainAction,
            MonitorEventType::MonitorStarted,
            MonitorEventType::MonitorStopped,
        ];
        for t in &types {
            let entry = MonitorLogEntry {
                timestamp: 1700000000,
                event_type: t.clone(),
                severity: None,
                process: None,
                pid: None,
                path: None,
                detail: "test".to_string(),
            };
            let json = format_log_entry(&entry).expect("serialize");
            let parsed: serde_json::Value = serde_json::from_str(&json).expect("parse");
            assert!(parsed["event_type"].is_string());
        }
    }
}
