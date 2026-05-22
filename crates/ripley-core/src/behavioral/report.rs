use serde::{Deserialize, Serialize};

use crate::types::{Ecosystem, Severity};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralReport {
    pub package: String,
    pub version: String,
    pub ecosystem: Ecosystem,
    pub declared: DeclaredBehavior,
    pub observed: ObservedBehavior,
    pub anomalies: Vec<BehavioralAnomaly>,
    pub risk_score: f64,
    pub analysis_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeclaredBehavior {
    pub has_install_scripts: bool,
    pub script_names: Vec<String>,
    pub declared_dependencies: Vec<String>,
    pub known_build_tool: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ObservedBehavior {
    pub network_attempts: Vec<NetworkAttempt>,
    pub fs_writes: Vec<FsWrite>,
    pub fs_reads: Vec<FsRead>,
    pub process_spawns: Vec<ProcessSpawn>,
    pub exit_code: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkAttempt {
    pub host: String,
    pub port: Option<u16>,
    pub blocked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsWrite {
    pub path: String,
    pub blocked: bool,
    pub outside_package: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsRead {
    pub path: String,
    pub sensitive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessSpawn {
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralAnomaly {
    pub kind: AnomalyKind,
    pub severity: Severity,
    pub description: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnomalyKind {
    UnexpectedNetwork,
    ScopeEscape,
    CredentialAccess,
    SuspiciousSpawn,
    BehaviorMismatch,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_behavioral_report_serialization() {
        let report = BehavioralReport {
            package: "evil-pkg".to_string(),
            version: "1.0.0".to_string(),
            ecosystem: Ecosystem::Npm,
            declared: DeclaredBehavior::default(),
            observed: ObservedBehavior::default(),
            anomalies: Vec::new(),
            risk_score: 0.0,
            analysis_duration_ms: 50,
        };
        let json = serde_json::to_string(&report).expect("serialize");
        assert!(json.contains("\"evil-pkg\""));
        assert!(json.contains("\"risk_score\":0.0"));
    }

    #[test]
    fn test_declared_behavior_defaults() {
        let declared = DeclaredBehavior::default();
        assert!(!declared.has_install_scripts);
        assert!(!declared.known_build_tool);
        assert!(declared.script_names.is_empty());
    }

    #[test]
    fn test_observed_behavior_defaults() {
        let observed = ObservedBehavior::default();
        assert_eq!(observed.exit_code, 0);
        assert!(observed.network_attempts.is_empty());
        assert!(observed.fs_writes.is_empty());
    }

    #[test]
    fn test_anomaly_kind_roundtrip() {
        let anomaly = BehavioralAnomaly {
            kind: AnomalyKind::UnexpectedNetwork,
            severity: Severity::High,
            description: "network from non-network pkg".to_string(),
            evidence: "curl example.com".to_string(),
        };
        let json = serde_json::to_string(&anomaly).expect("serialize");
        assert!(json.contains("\"unexpected_network\""));
        let roundtrip: BehavioralAnomaly = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(roundtrip.kind, AnomalyKind::UnexpectedNetwork);
    }

    #[test]
    fn test_network_attempt_serialization() {
        let attempt = NetworkAttempt {
            host: "evil.com".to_string(),
            port: Some(443),
            blocked: true,
        };
        let json = serde_json::to_string(&attempt).expect("serialize");
        assert!(json.contains("\"evil.com\""));
        assert!(json.contains("\"blocked\":true"));
    }

    #[test]
    fn test_fs_write_serialization() {
        let write = FsWrite {
            path: "/etc/passwd".to_string(),
            blocked: true,
            outside_package: true,
        };
        let json = serde_json::to_string(&write).expect("serialize");
        assert!(json.contains("\"outside_package\":true"));
    }
}
