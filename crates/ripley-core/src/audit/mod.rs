pub mod ai_tools;
pub mod creds;
pub mod machine;
pub mod toolchain;

use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrafficLight {
    Green,
    Yellow,
    Red,
}

impl fmt::Display for TrafficLight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrafficLight::Green => write!(f, "green"),
            TrafficLight::Yellow => write!(f, "yellow"),
            TrafficLight::Red => write!(f, "red"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditCategory {
    MachineSecurity,
    Toolchain,
    AiTools,
    Credentials,
}

impl fmt::Display for AuditCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuditCategory::MachineSecurity => write!(f, "Machine Security"),
            AuditCategory::Toolchain => write!(f, "Developer Toolchain"),
            AuditCategory::AiTools => write!(f, "AI Tool Config"),
            AuditCategory::Credentials => write!(f, "Credential Exposure"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditFinding {
    pub name: String,
    pub status: TrafficLight,
    pub detail: String,
    pub fix_command: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CategoryReport {
    pub category: AuditCategory,
    pub findings: Vec<AuditFinding>,
    pub overall: TrafficLight,
}

impl CategoryReport {
    pub fn new(category: AuditCategory, findings: Vec<AuditFinding>) -> Self {
        let overall = Self::compute_overall(&findings);
        Self {
            category,
            findings,
            overall,
        }
    }

    fn compute_overall(findings: &[AuditFinding]) -> TrafficLight {
        if findings.iter().any(|f| f.status == TrafficLight::Red) {
            TrafficLight::Red
        } else if findings.iter().any(|f| f.status == TrafficLight::Yellow) {
            TrafficLight::Yellow
        } else {
            TrafficLight::Green
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    pub categories: Vec<CategoryReport>,
    pub timestamp: String,
}

#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error("failed to run system command: {0}")]
    Command(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub fn run_audit(home: &std::path::Path) -> AuditReport {
    let categories = vec![
        machine::check_machine_security(),
        toolchain::check_toolchain(home),
        ai_tools::check_ai_tools(home),
        creds::check_credentials(home),
    ];

    AuditReport {
        categories,
        timestamp: chrono::Utc::now().to_rfc3339(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_traffic_light_ordering() {
        assert!(TrafficLight::Green < TrafficLight::Yellow);
        assert!(TrafficLight::Yellow < TrafficLight::Red);
        assert!(TrafficLight::Green < TrafficLight::Red);
    }

    #[test]
    fn test_category_report_overall_red() {
        let findings = vec![
            AuditFinding {
                name: "check1".to_string(),
                status: TrafficLight::Green,
                detail: "ok".to_string(),
                fix_command: None,
            },
            AuditFinding {
                name: "check2".to_string(),
                status: TrafficLight::Red,
                detail: "bad".to_string(),
                fix_command: Some("fix it".to_string()),
            },
        ];
        let report = CategoryReport::new(AuditCategory::MachineSecurity, findings);
        assert_eq!(report.overall, TrafficLight::Red);
    }

    #[test]
    fn test_category_report_overall_yellow() {
        let findings = vec![
            AuditFinding {
                name: "check1".to_string(),
                status: TrafficLight::Green,
                detail: "ok".to_string(),
                fix_command: None,
            },
            AuditFinding {
                name: "check2".to_string(),
                status: TrafficLight::Yellow,
                detail: "warning".to_string(),
                fix_command: None,
            },
        ];
        let report = CategoryReport::new(AuditCategory::Toolchain, findings);
        assert_eq!(report.overall, TrafficLight::Yellow);
    }

    #[test]
    fn test_category_report_overall_green() {
        let findings = vec![AuditFinding {
            name: "check1".to_string(),
            status: TrafficLight::Green,
            detail: "ok".to_string(),
            fix_command: None,
        }];
        let report = CategoryReport::new(AuditCategory::AiTools, findings);
        assert_eq!(report.overall, TrafficLight::Green);
    }

    #[test]
    fn test_category_report_overall_empty() {
        let report = CategoryReport::new(AuditCategory::Credentials, Vec::new());
        assert_eq!(report.overall, TrafficLight::Green);
    }

    #[test]
    fn test_traffic_light_serde_roundtrip() {
        let json = serde_json::to_string(&TrafficLight::Red).expect("serialize");
        assert_eq!(json, "\"red\"");
        let parsed: TrafficLight = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, TrafficLight::Red);
    }

    #[test]
    fn test_audit_category_display() {
        assert_eq!(
            AuditCategory::MachineSecurity.to_string(),
            "Machine Security"
        );
        assert_eq!(AuditCategory::Toolchain.to_string(), "Developer Toolchain");
        assert_eq!(AuditCategory::AiTools.to_string(), "AI Tool Config");
        assert_eq!(
            AuditCategory::Credentials.to_string(),
            "Credential Exposure"
        );
    }
}
