use std::time::Duration;

use serde::Deserialize;

use super::{Advisory, FeedError, FeedSource};
use crate::types::{Ecosystem, Severity};

const SOCKET_API_BASE: &str = "https://api.socket.dev/v0";

pub struct SocketClient {
    client: reqwest::Client,
    api_key: String,
}

// --- Response types (private) ---

#[derive(Deserialize)]
struct SocketResponse {
    #[serde(default)]
    alerts: Vec<SocketAlert>,
}

#[derive(Deserialize)]
struct SocketAlert {
    #[serde(rename = "type")]
    alert_type: String,
    #[serde(default)]
    severity: Option<String>,
    #[serde(default)]
    key: Option<String>,
}

// --- Implementation ---

impl SocketClient {
    pub fn new(api_key: String) -> Result<Self, FeedError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("ripley-supply-chain-guard")
            .build()?;

        Ok(Self { client, api_key })
    }

    pub async fn query_package(
        &self,
        ecosystem: Ecosystem,
        package: &str,
        version: &str,
    ) -> Result<Vec<Advisory>, FeedError> {
        let Some(eco_str) = ecosystem_to_socket(ecosystem) else {
            return Ok(Vec::new());
        };

        let mut url = reqwest::Url::parse(SOCKET_API_BASE)
            .map_err(|e| FeedError::Internal(format!("invalid Socket API base URL: {e}")))?;
        url.path_segments_mut()
            .map_err(|_| FeedError::Internal("cannot modify Socket API URL".into()))?
            .push(eco_str)
            .push(package)
            .push("score");

        let response = self
            .client
            .get(url)
            .header("Authorization", format!("Basic {}", self.api_key))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(FeedError::Internal(format!(
                "Socket.dev API returned status {}",
                response.status()
            )));
        }

        let socket_response: SocketResponse = response
            .json()
            .await
            .map_err(|e| FeedError::Parse(format!("failed to parse Socket.dev response: {e}")))?;

        let advisories = socket_response
            .alerts
            .into_iter()
            .map(|alert| map_alert(alert, ecosystem, package, version))
            .collect();

        Ok(advisories)
    }
}

fn ecosystem_to_socket(eco: Ecosystem) -> Option<&'static str> {
    match eco {
        Ecosystem::Npm => Some("npm"),
        Ecosystem::PyPI => Some("pypi"),
        Ecosystem::Cargo => Some("cargo"),
        Ecosystem::Go | Ecosystem::Gem => None,
    }
}

fn socket_severity_to_severity(severity: &str) -> Option<Severity> {
    match severity.to_lowercase().as_str() {
        "critical" => Some(Severity::Critical),
        "high" => Some(Severity::High),
        "medium" | "moderate" => Some(Severity::Medium),
        "low" => Some(Severity::Low),
        _ => None,
    }
}

fn map_alert(alert: SocketAlert, ecosystem: Ecosystem, package: &str, version: &str) -> Advisory {
    let severity = alert
        .severity
        .as_deref()
        .and_then(socket_severity_to_severity);

    let id = alert
        .key
        .unwrap_or_else(|| format!("socket-{}-{}-{}", package, alert.alert_type, version));

    Advisory {
        id,
        source: FeedSource::Socket,
        ecosystem,
        package: package.to_string(),
        affected_ranges: Vec::new(),
        severity,
        summary: alert.alert_type,
        references: vec![format!(
            "https://socket.dev/{}/package/{}/overview",
            ecosystem_to_socket(ecosystem).unwrap_or("npm"),
            package
        )],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MOCK_SOCKET_RESPONSE: &str = r#"{
  "alerts": [
    {
      "type": "malware",
      "severity": "critical",
      "key": "socket-alert-malware-001"
    },
    {
      "type": "typosquatting",
      "severity": "high",
      "key": "socket-alert-typosquat-002"
    },
    {
      "type": "installScripts",
      "severity": "medium",
      "key": "socket-alert-install-003"
    },
    {
      "type": "telemetry",
      "severity": "low",
      "key": null
    }
  ]
}"#;

    #[test]
    fn test_parse_response() {
        let response: SocketResponse =
            serde_json::from_str(MOCK_SOCKET_RESPONSE).expect("should parse mock response");

        assert_eq!(response.alerts.len(), 4);

        assert_eq!(response.alerts[0].alert_type, "malware");
        assert_eq!(response.alerts[0].severity.as_deref(), Some("critical"));
        assert_eq!(
            response.alerts[0].key.as_deref(),
            Some("socket-alert-malware-001")
        );

        assert_eq!(response.alerts[1].alert_type, "typosquatting");
        assert_eq!(response.alerts[1].severity.as_deref(), Some("high"));

        assert_eq!(response.alerts[2].alert_type, "installScripts");
        assert_eq!(response.alerts[2].severity.as_deref(), Some("medium"));

        assert_eq!(response.alerts[3].alert_type, "telemetry");
        assert_eq!(response.alerts[3].severity.as_deref(), Some("low"));
        assert!(response.alerts[3].key.is_none());

        // Verify mapping to Advisory structs
        let advisories: Vec<Advisory> = response
            .alerts
            .into_iter()
            .map(|a| map_alert(a, Ecosystem::Npm, "evil-package", "1.0.0"))
            .collect();

        assert_eq!(advisories.len(), 4);

        assert_eq!(advisories[0].id, "socket-alert-malware-001");
        assert_eq!(advisories[0].ecosystem, Ecosystem::Npm);
        assert_eq!(advisories[0].package, "evil-package");
        assert_eq!(advisories[0].summary, "malware");
        assert_eq!(advisories[0].severity, Some(Severity::Critical));
        assert!(advisories[0].affected_ranges.is_empty());
        assert_eq!(
            advisories[0].references[0],
            "https://socket.dev/npm/package/evil-package/overview"
        );

        assert_eq!(advisories[1].id, "socket-alert-typosquat-002");
        assert_eq!(advisories[1].severity, Some(Severity::High));

        assert_eq!(advisories[2].severity, Some(Severity::Medium));

        // Alert with null key should get a generated ID
        assert_eq!(advisories[3].severity, Some(Severity::Low));
        assert_eq!(advisories[3].id, "socket-evil-package-telemetry-1.0.0");
    }

    #[test]
    fn test_ecosystem_to_socket() {
        assert_eq!(ecosystem_to_socket(Ecosystem::Npm), Some("npm"));
        assert_eq!(ecosystem_to_socket(Ecosystem::PyPI), Some("pypi"));
        assert_eq!(ecosystem_to_socket(Ecosystem::Cargo), Some("cargo"));
        assert_eq!(ecosystem_to_socket(Ecosystem::Go), None);
        assert_eq!(ecosystem_to_socket(Ecosystem::Gem), None);
    }

    #[test]
    fn test_unsupported_ecosystem() {
        // Go and Gem are not supported by Socket.dev — ecosystem_to_socket returns None
        // and query_package should return an empty vec.
        assert!(ecosystem_to_socket(Ecosystem::Go).is_none());
        assert!(ecosystem_to_socket(Ecosystem::Gem).is_none());
    }

    #[test]
    fn test_severity_mapping() {
        assert_eq!(
            socket_severity_to_severity("critical"),
            Some(Severity::Critical)
        );
        assert_eq!(socket_severity_to_severity("high"), Some(Severity::High));
        assert_eq!(
            socket_severity_to_severity("medium"),
            Some(Severity::Medium)
        );
        assert_eq!(
            socket_severity_to_severity("moderate"),
            Some(Severity::Medium)
        );
        assert_eq!(socket_severity_to_severity("low"), Some(Severity::Low));
        assert_eq!(socket_severity_to_severity("unknown"), None);
        assert_eq!(socket_severity_to_severity(""), None);
        // Case insensitivity
        assert_eq!(
            socket_severity_to_severity("CRITICAL"),
            Some(Severity::Critical)
        );
        assert_eq!(socket_severity_to_severity("High"), Some(Severity::High));
    }

    #[test]
    fn test_empty_alerts() {
        let json = r#"{"alerts": []}"#;
        let response: SocketResponse = serde_json::from_str(json).expect("parse");
        assert!(response.alerts.is_empty());
    }

    #[test]
    fn test_missing_alerts_field() {
        let json = r#"{}"#;
        let response: SocketResponse = serde_json::from_str(json).expect("parse");
        assert!(response.alerts.is_empty());
    }

    #[test]
    fn test_alert_missing_optional_fields() {
        let json = r#"{
            "alerts": [
                {
                    "type": "networkAccess"
                }
            ]
        }"#;
        let response: SocketResponse = serde_json::from_str(json).expect("parse");
        assert_eq!(response.alerts.len(), 1);
        assert_eq!(response.alerts[0].alert_type, "networkAccess");
        assert!(response.alerts[0].severity.is_none());
        assert!(response.alerts[0].key.is_none());

        let advisory = map_alert(
            response.alerts.into_iter().next().expect("has one"),
            Ecosystem::PyPI,
            "sketchy-lib",
            "0.5.0",
        );
        assert!(advisory.severity.is_none());
        assert_eq!(advisory.id, "socket-sketchy-lib-networkAccess-0.5.0");
        assert_eq!(
            advisory.references[0],
            "https://socket.dev/pypi/package/sketchy-lib/overview"
        );
    }
}
