use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::{Advisory, AffectedRange, FeedError};
use crate::types::{Ecosystem, Severity};

const OSV_QUERY_URL: &str = "https://api.osv.dev/v1/query";
const OSV_BATCH_URL: &str = "https://api.osv.dev/v1/querybatch";
const MAX_BATCH_SIZE: usize = 1000;
const MAX_RETRIES: u32 = 5;
const BASE_RETRY_DELAY: Duration = Duration::from_secs(1);
const MAX_RETRY_DELAY: Duration = Duration::from_secs(300);

pub struct OsvClient {
    client: reqwest::Client,
    cache_dir: PathBuf,
}

#[derive(Serialize)]
struct OsvQuery {
    package: OsvPackage,
}

#[derive(Serialize)]
struct OsvPackage {
    name: String,
    ecosystem: String,
}

#[derive(Serialize)]
struct OsvBatchRequest {
    queries: Vec<OsvQuery>,
}

#[derive(Deserialize)]
struct OsvResponse {
    #[serde(default)]
    vulns: Vec<OsvVuln>,
}

#[derive(Deserialize)]
struct OsvBatchResponse {
    #[serde(default)]
    results: Vec<OsvBatchResult>,
}

#[derive(Deserialize)]
struct OsvBatchResult {
    #[serde(default)]
    vulns: Vec<OsvVuln>,
}

#[derive(Deserialize)]
struct OsvVuln {
    id: String,
    #[serde(default)]
    summary: String,
    #[serde(default)]
    affected: Vec<OsvAffected>,
    #[serde(default)]
    severity: Vec<OsvSeverity>,
    #[serde(default)]
    references: Vec<OsvReference>,
    #[serde(default)]
    database_specific: Option<OsvDatabaseSpecific>,
}

#[derive(Deserialize)]
struct OsvAffected {
    #[serde(default)]
    ranges: Vec<OsvRange>,
}

#[derive(Deserialize)]
struct OsvRange {
    #[serde(default)]
    events: Vec<OsvEvent>,
}

#[derive(Deserialize)]
struct OsvEvent {
    #[serde(default)]
    introduced: Option<String>,
    #[serde(default)]
    fixed: Option<String>,
}

#[derive(Deserialize)]
struct OsvSeverity {
    #[serde(default, rename = "type")]
    severity_type: Option<String>,
    #[serde(default)]
    score: Option<String>,
}

#[derive(Deserialize)]
struct OsvReference {
    #[serde(default)]
    url: String,
}

#[derive(Deserialize)]
struct OsvDatabaseSpecific {
    #[serde(default)]
    severity: Option<String>,
}

impl OsvClient {
    pub fn new(cache_dir: PathBuf) -> Result<Self, FeedError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;
        std::fs::create_dir_all(cache_dir.join("feeds")).map_err(FeedError::Io)?;
        Ok(Self { client, cache_dir })
    }

    pub async fn query(
        &self,
        ecosystem: Ecosystem,
        package: &str,
    ) -> Result<Vec<Advisory>, FeedError> {
        let osv_ecosystem = ecosystem_to_osv(ecosystem);
        let body = OsvQuery {
            package: OsvPackage {
                name: package.to_string(),
                ecosystem: osv_ecosystem.to_string(),
            },
        };

        let etag_path = self.cache_dir.join("feeds").join(format!(
            "osv_{}_{}.etag",
            osv_ecosystem,
            sanitize_name(package)
        ));

        let mut request = self.client.post(OSV_QUERY_URL).json(&body);

        if let Ok(etag) = std::fs::read_to_string(&etag_path) {
            request = request.header("If-None-Match", etag.trim());
        }

        let response = self.send_with_retry(request).await?;

        if response.status() == reqwest::StatusCode::NOT_MODIFIED {
            return Ok(Vec::new());
        }

        if let Some(etag) = response.headers().get("etag")
            && let Ok(etag_str) = etag.to_str()
        {
            let _ = std::fs::write(&etag_path, etag_str);
        }

        let osv_response: OsvResponse = response.json().await?;
        let advisories = osv_response
            .vulns
            .into_iter()
            .map(|v| map_vuln(v, ecosystem, package))
            .collect();

        Ok(advisories)
    }

    pub async fn query_batch(
        &self,
        queries: &[(Ecosystem, String)],
    ) -> Result<Vec<Advisory>, FeedError> {
        let mut all_advisories = Vec::new();

        for chunk in queries.chunks(MAX_BATCH_SIZE) {
            let osv_queries: Vec<OsvQuery> = chunk
                .iter()
                .map(|(eco, pkg)| OsvQuery {
                    package: OsvPackage {
                        name: pkg.clone(),
                        ecosystem: ecosystem_to_osv(*eco).to_string(),
                    },
                })
                .collect();

            let body = OsvBatchRequest {
                queries: osv_queries,
            };

            let request = self.client.post(OSV_BATCH_URL).json(&body);
            let response = self.send_with_retry(request).await?;
            let batch_response: OsvBatchResponse = response.json().await?;

            for (i, result) in batch_response.results.into_iter().enumerate() {
                let (ecosystem, package) = &chunk[i];
                for vuln in result.vulns {
                    all_advisories.push(map_vuln(vuln, *ecosystem, package));
                }
            }
        }

        Ok(all_advisories)
    }

    async fn send_with_retry(
        &self,
        request: reqwest::RequestBuilder,
    ) -> Result<reqwest::Response, FeedError> {
        let mut delay = BASE_RETRY_DELAY;

        for attempt in 0..=MAX_RETRIES {
            let req = request
                .try_clone()
                .ok_or_else(|| FeedError::Internal("could not clone request".to_string()))?;

            match req.send().await {
                Ok(resp) if resp.status().is_server_error() && attempt < MAX_RETRIES => {
                    let jitter = Duration::from_millis(rand_jitter());
                    tokio::time::sleep(delay + jitter).await;
                    delay = (delay * 2).min(MAX_RETRY_DELAY);
                }
                Ok(resp) => return Ok(resp),
                Err(e) if attempt < MAX_RETRIES && is_retryable(&e) => {
                    let jitter = Duration::from_millis(rand_jitter());
                    tokio::time::sleep(delay + jitter).await;
                    delay = (delay * 2).min(MAX_RETRY_DELAY);
                }
                Err(e) => return Err(FeedError::Http(e)),
            }
        }

        unreachable!()
    }
}

fn ecosystem_to_osv(eco: Ecosystem) -> &'static str {
    match eco {
        Ecosystem::Npm => "npm",
        Ecosystem::PyPI => "PyPI",
        Ecosystem::Cargo => "crates.io",
        Ecosystem::Go => "Go",
        Ecosystem::Gem => "RubyGems",
    }
}

fn sanitize_name(name: &str) -> String {
    name.replace('/', "_").replace('@', "")
}

fn map_vuln(vuln: OsvVuln, ecosystem: Ecosystem, package: &str) -> Advisory {
    let severity = extract_severity(&vuln.severity)
        .or_else(|| extract_database_severity(vuln.database_specific.as_ref()));

    let mut affected_ranges = Vec::new();
    for affected in &vuln.affected {
        for range in &affected.ranges {
            let mut introduced: Option<semver::Version> = None;
            for event in &range.events {
                if let Some(ref v) = event.introduced {
                    introduced = semver::Version::parse(v).ok();
                }
                if let Some(ref v) = event.fixed
                    && let Some(intro) = introduced.take()
                {
                    affected_ranges.push(AffectedRange {
                        introduced: intro,
                        fixed: semver::Version::parse(v).ok(),
                    });
                }
            }
            if let Some(intro) = introduced {
                affected_ranges.push(AffectedRange {
                    introduced: intro,
                    fixed: None,
                });
            }
        }
    }

    let references = vuln.references.into_iter().map(|r| r.url).collect();

    Advisory {
        id: vuln.id,
        ecosystem,
        package: package.to_string(),
        affected_ranges,
        severity,
        summary: vuln.summary,
        references,
    }
}

fn extract_severity(severity_list: &[OsvSeverity]) -> Option<Severity> {
    // Prefer CVSS_V3/V4 over V2
    let preferred = severity_list
        .iter()
        .find(|s| {
            s.severity_type
                .as_deref()
                .is_some_and(|t| t == "CVSS_V3" || t == "CVSS_V4")
        })
        .or_else(|| severity_list.first());

    preferred.and_then(|s| {
        s.score.as_ref().and_then(|score| {
            // Try parsing as bare numeric score first
            if let Ok(val) = score.parse::<f64>() {
                return Some(cvss_to_severity(val));
            }
            // Try extracting base score from CVSS vector string
            // Vectors look like: "CVSS:3.1/AV:N/AC:L/.../baseScore:7.5" or
            // have the score as the last numeric after a slash
            extract_score_from_vector(score).map(cvss_to_severity)
        })
    })
}

fn extract_score_from_vector(vector: &str) -> Option<f64> {
    // CVSS vectors don't embed the score — the score is computed from the vector.
    // But some OSV entries put just the score in the score field, while others
    // put the full vector. We can't fully compute CVSS from a vector without a
    // library, but we can handle the common case where entries report it as a
    // bare "7.5" or as part of a non-standard format.
    //
    // For proper CVSS vectors (CVSS:3.1/AV:N/...), return None and fall through
    // to database_specific.
    if vector.starts_with("CVSS:") {
        return None;
    }
    vector.parse::<f64>().ok()
}

fn extract_database_severity(db_specific: Option<&OsvDatabaseSpecific>) -> Option<Severity> {
    db_specific.and_then(|d| {
        d.severity
            .as_deref()
            .and_then(|s| match s.to_uppercase().as_str() {
                "CRITICAL" => Some(Severity::Critical),
                "HIGH" => Some(Severity::High),
                "MODERATE" | "MEDIUM" => Some(Severity::Medium),
                "LOW" => Some(Severity::Low),
                _ => None,
            })
    })
}

fn cvss_to_severity(score: f64) -> Severity {
    if score >= 9.0 {
        Severity::Critical
    } else if score >= 7.0 {
        Severity::High
    } else if score >= 4.0 {
        Severity::Medium
    } else {
        Severity::Low
    }
}

fn is_retryable(err: &reqwest::Error) -> bool {
    err.is_timeout() || err.is_connect()
}

fn rand_jitter() -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    std::time::SystemTime::now().hash(&mut hasher);
    std::thread::current().id().hash(&mut hasher);
    hasher.finish() % 500
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_osv_vuln() -> OsvVuln {
        OsvVuln {
            id: "GHSA-xxxx-yyyy-zzzz".to_string(),
            summary: "Prototype pollution in lodash".to_string(),
            affected: vec![OsvAffected {
                ranges: vec![OsvRange {
                    events: vec![
                        OsvEvent {
                            introduced: Some("0.0.0".to_string()),
                            fixed: None,
                        },
                        OsvEvent {
                            introduced: None,
                            fixed: Some("4.17.21".to_string()),
                        },
                    ],
                }],
            }],
            severity: vec![OsvSeverity {
                severity_type: Some("CVSS_V3".to_string()),
                score: Some("7.5".to_string()),
            }],
            references: vec![OsvReference {
                url: "https://github.com/advisories/GHSA-xxxx-yyyy-zzzz".to_string(),
            }],
            database_specific: None,
        }
    }

    #[test]
    fn test_parse_osv_response() {
        let json = r#"{
            "vulns": [
                {
                    "id": "GHSA-test-1234-abcd",
                    "summary": "Test vulnerability",
                    "affected": [
                        {
                            "ranges": [
                                {
                                    "events": [
                                        {"introduced": "1.0.0"},
                                        {"fixed": "1.0.5"}
                                    ]
                                }
                            ]
                        }
                    ],
                    "severity": [{"type": "CVSS_V3", "score": "9.1"}],
                    "references": [{"url": "https://example.com"}]
                }
            ]
        }"#;

        let response: OsvResponse = serde_json::from_str(json).expect("parse");
        assert_eq!(response.vulns.len(), 1);
        assert_eq!(response.vulns[0].id, "GHSA-test-1234-abcd");
    }

    #[test]
    fn test_advisory_mapping() {
        let vuln = sample_osv_vuln();
        let advisory = map_vuln(vuln, Ecosystem::Npm, "lodash");

        assert_eq!(advisory.id, "GHSA-xxxx-yyyy-zzzz");
        assert_eq!(advisory.ecosystem, Ecosystem::Npm);
        assert_eq!(advisory.package, "lodash");
        assert_eq!(advisory.affected_ranges.len(), 1);
        assert_eq!(
            advisory.affected_ranges[0].introduced,
            semver::Version::new(0, 0, 0)
        );
        assert_eq!(
            advisory.affected_ranges[0].fixed,
            Some(semver::Version::new(4, 17, 21))
        );
        assert_eq!(advisory.severity, Some(Severity::High));
        assert_eq!(advisory.references.len(), 1);
    }

    #[test]
    fn test_cvss_to_severity() {
        assert_eq!(cvss_to_severity(9.5), Severity::Critical);
        assert_eq!(cvss_to_severity(7.0), Severity::High);
        assert_eq!(cvss_to_severity(4.0), Severity::Medium);
        assert_eq!(cvss_to_severity(2.0), Severity::Low);
    }

    #[test]
    fn test_ecosystem_to_osv() {
        assert_eq!(ecosystem_to_osv(Ecosystem::Npm), "npm");
        assert_eq!(ecosystem_to_osv(Ecosystem::PyPI), "PyPI");
        assert_eq!(ecosystem_to_osv(Ecosystem::Cargo), "crates.io");
    }

    #[test]
    fn test_sanitize_name() {
        assert_eq!(
            sanitize_name("@tanstack/react-query"),
            "tanstack_react-query"
        );
        assert_eq!(sanitize_name("lodash"), "lodash");
    }

    #[test]
    fn test_open_ended_affected_range() {
        let vuln = OsvVuln {
            id: "TEST-001".to_string(),
            summary: "Open-ended range".to_string(),
            affected: vec![OsvAffected {
                ranges: vec![OsvRange {
                    events: vec![OsvEvent {
                        introduced: Some("2.0.0".to_string()),
                        fixed: None,
                    }],
                }],
            }],
            severity: Vec::new(),
            references: Vec::new(),
            database_specific: None,
        };

        let advisory = map_vuln(vuln, Ecosystem::Npm, "test-pkg");
        assert_eq!(advisory.affected_ranges.len(), 1);
        assert!(advisory.affected_ranges[0].fixed.is_none());
    }

    #[test]
    fn test_severity_from_cvss_vector_falls_through_to_database_specific() {
        let vuln = OsvVuln {
            id: "GHSA-vec-test".to_string(),
            summary: "Vector string severity".to_string(),
            affected: Vec::new(),
            severity: vec![OsvSeverity {
                severity_type: Some("CVSS_V3".to_string()),
                score: Some("CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H".to_string()),
            }],
            references: Vec::new(),
            database_specific: Some(OsvDatabaseSpecific {
                severity: Some("CRITICAL".to_string()),
            }),
        };

        let advisory = map_vuln(vuln, Ecosystem::Npm, "test-pkg");
        assert_eq!(advisory.severity, Some(Severity::Critical));
    }

    #[test]
    fn test_severity_from_database_specific_moderate() {
        let vuln = OsvVuln {
            id: "GHSA-mod-test".to_string(),
            summary: "Moderate severity".to_string(),
            affected: Vec::new(),
            severity: Vec::new(),
            references: Vec::new(),
            database_specific: Some(OsvDatabaseSpecific {
                severity: Some("MODERATE".to_string()),
            }),
        };

        let advisory = map_vuln(vuln, Ecosystem::Npm, "test-pkg");
        assert_eq!(advisory.severity, Some(Severity::Medium));
    }

    #[test]
    fn test_severity_prefers_numeric_score() {
        let vuln = OsvVuln {
            id: "GHSA-num-test".to_string(),
            summary: "Numeric score".to_string(),
            affected: Vec::new(),
            severity: vec![OsvSeverity {
                severity_type: Some("CVSS_V3".to_string()),
                score: Some("9.8".to_string()),
            }],
            references: Vec::new(),
            database_specific: Some(OsvDatabaseSpecific {
                severity: Some("LOW".to_string()),
            }),
        };

        let advisory = map_vuln(vuln, Ecosystem::Npm, "test-pkg");
        assert_eq!(advisory.severity, Some(Severity::Critical));
    }
}
