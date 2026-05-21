use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::{Advisory, AffectedRange, FeedError, FeedSource};
use crate::types::{Ecosystem, Severity};

const GITHUB_GRAPHQL_URL: &str = "https://api.github.com/graphql";
const PAGE_SIZE: u32 = 100;
const MAX_PAGES: u32 = 10;

pub struct GhsaClient {
    client: reqwest::Client,
    token: Option<String>,
}

const QUERY: &str = r#"
query($ecosystem: SecurityAdvisoryEcosystem!, $package: String!, $first: Int!, $after: String) {
  securityVulnerabilities(
    ecosystem: $ecosystem,
    package: $package,
    first: $first,
    after: $after
  ) {
    pageInfo {
      hasNextPage
      endCursor
    }
    nodes {
      advisory {
        ghsaId
        summary
        severity
        references {
          url
        }
      }
      package {
        name
      }
      vulnerableVersionRange
      firstPatchedVersion {
        identifier
      }
    }
  }
}
"#;

// --- GraphQL request/response types (private) ---

#[derive(Serialize)]
struct GraphQlRequest {
    query: &'static str,
    variables: QueryVariables,
}

#[derive(Serialize)]
struct QueryVariables {
    ecosystem: String,
    package: String,
    first: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    after: Option<String>,
}

#[derive(Deserialize)]
struct GraphQlResponse {
    data: Option<GraphQlData>,
    #[serde(default)]
    errors: Vec<GraphQlError>,
}

#[derive(Deserialize)]
struct GraphQlError {
    message: String,
}

#[derive(Deserialize)]
struct GraphQlData {
    #[serde(rename = "securityVulnerabilities")]
    security_vulnerabilities: SecurityVulnerabilities,
}

#[derive(Deserialize)]
struct SecurityVulnerabilities {
    #[serde(rename = "pageInfo")]
    page_info: PageInfo,
    nodes: Vec<VulnerabilityNode>,
}

#[derive(Deserialize)]
struct PageInfo {
    #[serde(rename = "hasNextPage")]
    has_next_page: bool,
    #[serde(rename = "endCursor")]
    end_cursor: Option<String>,
}

#[derive(Deserialize)]
struct VulnerabilityNode {
    advisory: AdvisoryNode,
    package: PackageNode,
    #[serde(rename = "vulnerableVersionRange")]
    vulnerable_version_range: Option<String>,
    #[serde(rename = "firstPatchedVersion")]
    first_patched_version: Option<PatchedVersion>,
}

#[derive(Deserialize)]
struct AdvisoryNode {
    #[serde(rename = "ghsaId")]
    ghsa_id: String,
    summary: Option<String>,
    severity: String,
    references: Vec<ReferenceNode>,
}

#[derive(Deserialize)]
struct ReferenceNode {
    url: String,
}

#[derive(Deserialize)]
struct PackageNode {
    name: String,
}

#[derive(Deserialize)]
struct PatchedVersion {
    identifier: String,
}

// --- Implementation ---

impl GhsaClient {
    pub fn new(token: Option<String>) -> Result<Self, FeedError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("ripley-supply-chain-guard")
            .build()?;

        Ok(Self { client, token })
    }

    pub async fn query_package(
        &self,
        ecosystem: Ecosystem,
        package: &str,
    ) -> Result<Vec<Advisory>, FeedError> {
        let ghsa_ecosystem = ecosystem_to_ghsa(ecosystem);
        let mut all_advisories = Vec::new();
        let mut cursor: Option<String> = None;

        for _ in 0..MAX_PAGES {
            let body = GraphQlRequest {
                query: QUERY,
                variables: QueryVariables {
                    ecosystem: ghsa_ecosystem.to_string(),
                    package: package.to_string(),
                    first: PAGE_SIZE,
                    after: cursor.clone(),
                },
            };

            let mut request = self.client.post(GITHUB_GRAPHQL_URL).json(&body);

            if let Some(ref token) = self.token {
                request = request.bearer_auth(token);
            }

            let response = request.send().await?;

            if !response.status().is_success() {
                return Err(FeedError::Internal(format!(
                    "GitHub GraphQL API returned status {}",
                    response.status()
                )));
            }

            let graphql_response: GraphQlResponse = response.json().await?;

            if let Some(err) = graphql_response.errors.first() {
                return Err(FeedError::Parse(format!("GraphQL error: {}", err.message)));
            }

            let data = graphql_response
                .data
                .ok_or_else(|| FeedError::Parse("missing data in GraphQL response".to_string()))?;

            let vulns = data.security_vulnerabilities;

            for node in &vulns.nodes {
                all_advisories.push(map_node(node, ecosystem));
            }

            if vulns.page_info.has_next_page {
                cursor = vulns.page_info.end_cursor;
            } else {
                break;
            }
        }

        Ok(all_advisories)
    }
}

fn ecosystem_to_ghsa(eco: Ecosystem) -> &'static str {
    match eco {
        Ecosystem::Npm => "NPM",
        Ecosystem::PyPI => "PIP",
        Ecosystem::Cargo => "RUST",
        Ecosystem::Go => "GO",
        Ecosystem::Gem => "RUBYGEMS",
    }
}

fn ghsa_severity_to_severity(severity: &str) -> Option<Severity> {
    match severity.to_uppercase().as_str() {
        "CRITICAL" => Some(Severity::Critical),
        "HIGH" => Some(Severity::High),
        "MODERATE" => Some(Severity::Medium),
        "LOW" => Some(Severity::Low),
        _ => None,
    }
}

fn map_node(node: &VulnerabilityNode, ecosystem: Ecosystem) -> Advisory {
    let affected_ranges = parse_affected_range(
        node.vulnerable_version_range.as_deref(),
        node.first_patched_version.as_ref(),
    );

    let references = node
        .advisory
        .references
        .iter()
        .map(|r| r.url.clone())
        .collect();

    Advisory {
        id: node.advisory.ghsa_id.clone(),
        source: FeedSource::Ghsa,
        ecosystem,
        package: node.package.name.clone(),
        affected_ranges,
        severity: ghsa_severity_to_severity(&node.advisory.severity),
        summary: node.advisory.summary.clone().unwrap_or_default(),
        references,
    }
}

/// Parse GHSA `vulnerableVersionRange` into `AffectedRange` entries.
///
/// GHSA version ranges use a comma-separated format, e.g.:
/// - `">= 1.0.0, < 2.0.0"` — introduced at 1.0.0, fixed at 2.0.0
/// - `"< 1.5.0"` — introduced at 0.0.0, fixed at 1.5.0
/// - `">= 3.0.0"` — introduced at 3.0.0, no fix
/// - `"= 1.2.3"` — exact version, introduced at 1.2.3, no fix
///
/// When `firstPatchedVersion` is available, it takes precedence as the fixed version.
fn parse_affected_range(
    version_range: Option<&str>,
    first_patched: Option<&PatchedVersion>,
) -> Vec<AffectedRange> {
    let Some(range_str) = version_range else {
        return Vec::new();
    };

    let parts: Vec<&str> = range_str.split(',').map(|s| s.trim()).collect();

    let mut introduced: Option<semver::Version> = None;
    let mut fixed: Option<semver::Version> = None;

    for part in &parts {
        if let Some(version_str) = part.strip_prefix(">=") {
            let version_str = version_str.trim();
            if let Ok(v) = semver::Version::parse(version_str) {
                introduced = Some(v);
            }
        } else if let Some(version_str) = part.strip_prefix('>') {
            let version_str = version_str.trim();
            if let Ok(v) = semver::Version::parse(version_str) {
                introduced = Some(v);
            }
        } else if let Some(version_str) = part.strip_prefix("<=") {
            // Upper bound inclusive — treat as fixed at next version (best effort)
            let version_str = version_str.trim();
            let _ = semver::Version::parse(version_str); // acknowledged but we prefer firstPatchedVersion
        } else if let Some(version_str) = part.strip_prefix('<') {
            let version_str = version_str.trim();
            if let Ok(v) = semver::Version::parse(version_str) {
                fixed = Some(v);
            }
        } else if let Some(version_str) = part.strip_prefix('=') {
            let version_str = version_str.trim();
            if let Ok(v) = semver::Version::parse(version_str) {
                introduced = Some(v);
            }
        }
    }

    // If no lower bound was found, assume 0.0.0
    if introduced.is_none() && fixed.is_some() {
        introduced = Some(semver::Version::new(0, 0, 0));
    }

    // firstPatchedVersion takes precedence over the range upper bound
    if let Some(patched) = first_patched
        && let Ok(v) = semver::Version::parse(&patched.identifier)
    {
        fixed = Some(v);
    }

    match introduced {
        Some(intro) => vec![AffectedRange {
            introduced: intro,
            fixed,
        }],
        None => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MOCK_GRAPHQL_RESPONSE: &str = r#"{
  "data": {
    "securityVulnerabilities": {
      "pageInfo": {
        "hasNextPage": false,
        "endCursor": "Y3Vyc29yOnYyOpHOBGD2Cg=="
      },
      "nodes": [
        {
          "advisory": {
            "ghsaId": "GHSA-35jh-r3h4-6jhm",
            "summary": "Prototype Pollution in lodash",
            "severity": "CRITICAL",
            "references": [
              {
                "url": "https://github.com/advisories/GHSA-35jh-r3h4-6jhm"
              },
              {
                "url": "https://nvd.nist.gov/vuln/detail/CVE-2020-8203"
              }
            ]
          },
          "package": {
            "name": "lodash"
          },
          "vulnerableVersionRange": ">= 4.0.0, < 4.17.20",
          "firstPatchedVersion": {
            "identifier": "4.17.20"
          }
        },
        {
          "advisory": {
            "ghsaId": "GHSA-jf85-cpcp-j695",
            "summary": "Command Injection in lodash",
            "severity": "HIGH",
            "references": [
              {
                "url": "https://github.com/advisories/GHSA-jf85-cpcp-j695"
              }
            ]
          },
          "package": {
            "name": "lodash"
          },
          "vulnerableVersionRange": "< 4.17.21",
          "firstPatchedVersion": {
            "identifier": "4.17.21"
          }
        }
      ]
    }
  }
}"#;

    #[test]
    fn test_parse_graphql_response() {
        let response: GraphQlResponse =
            serde_json::from_str(MOCK_GRAPHQL_RESPONSE).expect("should parse mock response");

        let data = response.data.expect("should have data");
        let vulns = &data.security_vulnerabilities;

        assert_eq!(vulns.nodes.len(), 2);
        assert!(!vulns.page_info.has_next_page);
        assert_eq!(
            vulns.page_info.end_cursor.as_deref(),
            Some("Y3Vyc29yOnYyOpHOBGD2Cg==")
        );

        assert_eq!(vulns.nodes[0].advisory.ghsa_id, "GHSA-35jh-r3h4-6jhm");
        assert_eq!(
            vulns.nodes[0].advisory.summary.as_deref(),
            Some("Prototype Pollution in lodash")
        );
        assert_eq!(vulns.nodes[0].advisory.severity, "CRITICAL");
        assert_eq!(vulns.nodes[0].advisory.references.len(), 2);
        assert_eq!(vulns.nodes[0].package.name, "lodash");
        assert_eq!(
            vulns.nodes[0].vulnerable_version_range.as_deref(),
            Some(">= 4.0.0, < 4.17.20")
        );
        assert_eq!(
            vulns.nodes[0]
                .first_patched_version
                .as_ref()
                .map(|v| v.identifier.as_str()),
            Some("4.17.20")
        );

        assert_eq!(vulns.nodes[1].advisory.ghsa_id, "GHSA-jf85-cpcp-j695");
        assert_eq!(vulns.nodes[1].advisory.severity, "HIGH");
    }

    #[test]
    fn test_advisory_mapping() {
        let response: GraphQlResponse =
            serde_json::from_str(MOCK_GRAPHQL_RESPONSE).expect("should parse");

        let data = response.data.expect("should have data");
        let nodes = &data.security_vulnerabilities.nodes;

        let advisory = map_node(&nodes[0], Ecosystem::Npm);

        assert_eq!(advisory.id, "GHSA-35jh-r3h4-6jhm");
        assert_eq!(advisory.ecosystem, Ecosystem::Npm);
        assert_eq!(advisory.package, "lodash");
        assert_eq!(advisory.summary, "Prototype Pollution in lodash");
        assert_eq!(advisory.severity, Some(Severity::Critical));
        assert_eq!(advisory.references.len(), 2);
        assert_eq!(
            advisory.references[0],
            "https://github.com/advisories/GHSA-35jh-r3h4-6jhm"
        );

        assert_eq!(advisory.affected_ranges.len(), 1);
        assert_eq!(
            advisory.affected_ranges[0].introduced,
            semver::Version::new(4, 0, 0)
        );
        assert_eq!(
            advisory.affected_ranges[0].fixed,
            Some(semver::Version::new(4, 17, 20))
        );

        // Second advisory: no lower bound in range, should default to 0.0.0
        let advisory2 = map_node(&nodes[1], Ecosystem::Npm);

        assert_eq!(advisory2.id, "GHSA-jf85-cpcp-j695");
        assert_eq!(advisory2.severity, Some(Severity::High));
        assert_eq!(advisory2.affected_ranges.len(), 1);
        assert_eq!(
            advisory2.affected_ranges[0].introduced,
            semver::Version::new(0, 0, 0)
        );
        assert_eq!(
            advisory2.affected_ranges[0].fixed,
            Some(semver::Version::new(4, 17, 21))
        );
    }

    #[test]
    fn test_ecosystem_to_ghsa() {
        assert_eq!(ecosystem_to_ghsa(Ecosystem::Npm), "NPM");
        assert_eq!(ecosystem_to_ghsa(Ecosystem::PyPI), "PIP");
        assert_eq!(ecosystem_to_ghsa(Ecosystem::Cargo), "RUST");
        assert_eq!(ecosystem_to_ghsa(Ecosystem::Go), "GO");
        assert_eq!(ecosystem_to_ghsa(Ecosystem::Gem), "RUBYGEMS");
    }

    #[test]
    fn test_severity_mapping() {
        assert_eq!(
            ghsa_severity_to_severity("CRITICAL"),
            Some(Severity::Critical)
        );
        assert_eq!(ghsa_severity_to_severity("HIGH"), Some(Severity::High));
        assert_eq!(
            ghsa_severity_to_severity("MODERATE"),
            Some(Severity::Medium)
        );
        assert_eq!(ghsa_severity_to_severity("LOW"), Some(Severity::Low));
        assert_eq!(ghsa_severity_to_severity("UNKNOWN"), None);
        // Case insensitivity
        assert_eq!(
            ghsa_severity_to_severity("critical"),
            Some(Severity::Critical)
        );
        assert_eq!(ghsa_severity_to_severity("High"), Some(Severity::High));
    }

    #[test]
    fn test_parse_range_with_bounds() {
        let ranges = parse_affected_range(Some(">= 1.0.0, < 2.0.0"), None);
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].introduced, semver::Version::new(1, 0, 0));
        assert_eq!(ranges[0].fixed, Some(semver::Version::new(2, 0, 0)));
    }

    #[test]
    fn test_parse_range_upper_only() {
        let ranges = parse_affected_range(Some("< 1.5.0"), None);
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].introduced, semver::Version::new(0, 0, 0));
        assert_eq!(ranges[0].fixed, Some(semver::Version::new(1, 5, 0)));
    }

    #[test]
    fn test_parse_range_lower_only() {
        let ranges = parse_affected_range(Some(">= 3.0.0"), None);
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].introduced, semver::Version::new(3, 0, 0));
        assert!(ranges[0].fixed.is_none());
    }

    #[test]
    fn test_parse_range_exact_version() {
        let ranges = parse_affected_range(Some("= 1.2.3"), None);
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].introduced, semver::Version::new(1, 2, 3));
        assert!(ranges[0].fixed.is_none());
    }

    #[test]
    fn test_parse_range_first_patched_overrides() {
        let patched = PatchedVersion {
            identifier: "2.0.1".to_string(),
        };
        let ranges = parse_affected_range(Some(">= 1.0.0, < 2.0.0"), Some(&patched));
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].introduced, semver::Version::new(1, 0, 0));
        assert_eq!(ranges[0].fixed, Some(semver::Version::new(2, 0, 1)));
    }

    #[test]
    fn test_parse_range_none() {
        let ranges = parse_affected_range(None, None);
        assert!(ranges.is_empty());
    }

    #[test]
    fn test_graphql_errors_parsed() {
        let json = r#"{
            "errors": [
                {"message": "Field 'securityVulnerabilities' requires an argument: 'ecosystem'"}
            ]
        }"#;

        let response: GraphQlResponse = serde_json::from_str(json).expect("should parse errors");
        assert!(response.data.is_none());
        assert_eq!(response.errors.len(), 1);
        assert!(response.errors[0].message.contains("ecosystem"));
    }
}
