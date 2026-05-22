use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::analyzer::AnalysisResult;
use crate::behavioral::BehavioralReport;
use crate::lockfile::{LockfileWarning, RiskySpec};
use crate::matcher::Match;
use crate::types::Severity;

pub const SARIF_VERSION: &str = "2.1.0";
pub const SARIF_SCHEMA: &str = "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/main/sarif-2.1/schema/sarif-schema-2.1.0.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifLog {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub version: String,
    pub runs: Vec<SarifRun>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifRun {
    pub tool: SarifTool,
    pub results: Vec<SarifResult>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub invocations: Vec<SarifInvocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifTool {
    pub driver: SarifToolComponent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifToolComponent {
    pub name: String,
    pub version: String,
    pub semantic_version: String,
    pub information_uri: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<SarifReportingDescriptor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifReportingDescriptor {
    pub id: String,
    pub name: String,
    pub short_description: SarifMessage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_description: Option<SarifMessage>,
    pub default_configuration: SarifConfiguration,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help_uri: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifConfiguration {
    pub level: SarifLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SarifLevel {
    Error,
    Warning,
    Note,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifResult {
    pub rule_id: String,
    pub rule_index: usize,
    pub level: SarifLevel,
    pub message: SarifMessage,
    pub locations: Vec<SarifLocation>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub fingerprints: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifLocation {
    pub physical_location: SarifPhysicalLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifPhysicalLocation {
    pub artifact_location: SarifArtifactLocation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<SarifRegion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifArtifactLocation {
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri_base_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifRegion {
    pub start_line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_column: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifMessage {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifInvocation {
    pub execution_successful: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
}

pub fn severity_to_sarif_level(severity: &Severity) -> SarifLevel {
    match severity {
        Severity::Critical | Severity::High => SarifLevel::Error,
        Severity::Medium => SarifLevel::Warning,
        Severity::Low => SarifLevel::Note,
    }
}

pub fn new_sarif_log(runs: Vec<SarifRun>) -> SarifLog {
    SarifLog {
        schema: SARIF_SCHEMA.to_string(),
        version: SARIF_VERSION.to_string(),
        runs,
    }
}

pub fn ripley_tool_component(rules: Vec<SarifReportingDescriptor>) -> SarifToolComponent {
    SarifToolComponent {
        name: "ripley".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        semantic_version: env!("CARGO_PKG_VERSION").to_string(),
        information_uri: "https://github.com/agentstation/ripley".to_string(),
        rules,
    }
}

pub fn sarif_fingerprint(advisory_id: &str, package: &str, version: &str) -> String {
    format!("{advisory_id}:{package}@{version}")
}

pub fn matches_to_sarif(
    matches: &[Match],
    warnings: &[LockfileWarning],
    risky_specs: &[RiskySpec],
) -> SarifLog {
    let mut rules: Vec<SarifReportingDescriptor> = Vec::new();
    let mut rule_index_map: HashMap<String, usize> = HashMap::new();
    let mut results: Vec<SarifResult> = Vec::new();

    for m in matches {
        let rule_idx = *rule_index_map
            .entry(m.advisory.id.clone())
            .or_insert_with(|| {
                let idx = rules.len();
                let level = m
                    .advisory
                    .severity
                    .as_ref()
                    .map(severity_to_sarif_level)
                    .unwrap_or(SarifLevel::Warning);
                rules.push(SarifReportingDescriptor {
                    id: m.advisory.id.clone(),
                    name: m.advisory.id.clone(),
                    short_description: SarifMessage {
                        text: m.advisory.summary.clone(),
                    },
                    full_description: None,
                    default_configuration: SarifConfiguration { level },
                    help_uri: m.advisory.references.first().cloned(),
                });
                idx
            });

        let level = m
            .advisory
            .severity
            .as_ref()
            .map(severity_to_sarif_level)
            .unwrap_or(SarifLevel::Warning);

        let lockfile_uri = m
            .project_path
            .join("package-lock.json")
            .to_string_lossy()
            .to_string();

        let fingerprint = sarif_fingerprint(
            &m.advisory.id,
            &m.package.name,
            &m.package.version.to_string(),
        );

        results.push(SarifResult {
            rule_id: m.advisory.id.clone(),
            rule_index: rule_idx,
            level,
            message: SarifMessage {
                text: format!(
                    "{}@{} is affected by {}: {}",
                    m.package.name, m.package.version, m.advisory.id, m.advisory.summary
                ),
            },
            locations: vec![SarifLocation {
                physical_location: SarifPhysicalLocation {
                    artifact_location: SarifArtifactLocation {
                        uri: lockfile_uri,
                        uri_base_id: None,
                    },
                    region: None,
                },
            }],
            fingerprints: HashMap::from([("ripley/vulnerability/v1".to_string(), fingerprint)]),
        });
    }

    for w in warnings {
        let rule_id = format!("ripley/posture/{}", w.field);
        let rule_idx = *rule_index_map.entry(rule_id.clone()).or_insert_with(|| {
            let idx = rules.len();
            rules.push(SarifReportingDescriptor {
                id: rule_id.clone(),
                name: rule_id.clone(),
                short_description: SarifMessage {
                    text: format!("Lockfile posture: {}", w.field),
                },
                full_description: None,
                default_configuration: SarifConfiguration {
                    level: severity_to_sarif_level(&w.severity),
                },
                help_uri: None,
            });
            idx
        });

        results.push(SarifResult {
            rule_id: rule_id.clone(),
            rule_index: rule_idx,
            level: severity_to_sarif_level(&w.severity),
            message: SarifMessage {
                text: format!("{}: {}", w.package, w.message),
            },
            locations: Vec::new(),
            fingerprints: HashMap::new(),
        });
    }

    for r in risky_specs {
        let rule_id = "ripley/posture/risky-spec".to_string();
        let rule_idx = *rule_index_map.entry(rule_id.clone()).or_insert_with(|| {
            let idx = rules.len();
            rules.push(SarifReportingDescriptor {
                id: rule_id.clone(),
                name: rule_id.clone(),
                short_description: SarifMessage {
                    text: "Risky version specifier".to_string(),
                },
                full_description: None,
                default_configuration: SarifConfiguration {
                    level: SarifLevel::Warning,
                },
                help_uri: None,
            });
            idx
        });

        results.push(SarifResult {
            rule_id: rule_id.clone(),
            rule_index: rule_idx,
            level: SarifLevel::Warning,
            message: SarifMessage {
                text: format!("{}: {} ({})", r.package, r.specifier, r.reason),
            },
            locations: Vec::new(),
            fingerprints: HashMap::new(),
        });
    }

    new_sarif_log(vec![SarifRun {
        tool: SarifTool {
            driver: ripley_tool_component(rules),
        },
        results,
        invocations: Vec::new(),
    }])
}

pub fn analysis_to_sarif(results: &[AnalysisResult], script_paths: &[PathBuf]) -> SarifLog {
    let mut rules: Vec<SarifReportingDescriptor> = Vec::new();
    let mut rule_index_map: HashMap<String, usize> = HashMap::new();
    let mut sarif_results: Vec<SarifResult> = Vec::new();

    for (result, path) in results.iter().zip(script_paths.iter()) {
        for matched in &result.matched_rules {
            let rule_idx = *rule_index_map
                .entry(matched.rule_id.clone())
                .or_insert_with(|| {
                    let idx = rules.len();
                    rules.push(SarifReportingDescriptor {
                        id: matched.rule_id.clone(),
                        name: matched.rule_name.clone(),
                        short_description: SarifMessage {
                            text: matched.rule_name.clone(),
                        },
                        full_description: None,
                        default_configuration: SarifConfiguration {
                            level: severity_to_sarif_level(&result.risk_level),
                        },
                        help_uri: None,
                    });
                    idx
                });

            sarif_results.push(SarifResult {
                rule_id: matched.rule_id.clone(),
                rule_index: rule_idx,
                level: severity_to_sarif_level(&result.risk_level),
                message: SarifMessage {
                    text: format!(
                        "Rule '{}' matched: {}",
                        matched.rule_name, matched.matched_text
                    ),
                },
                locations: vec![SarifLocation {
                    physical_location: SarifPhysicalLocation {
                        artifact_location: SarifArtifactLocation {
                            uri: path.to_string_lossy().to_string(),
                            uri_base_id: None,
                        },
                        region: Some(SarifRegion {
                            start_line: matched.matched_line,
                            start_column: None,
                        }),
                    },
                }],
                fingerprints: HashMap::new(),
            });
        }
    }

    new_sarif_log(vec![SarifRun {
        tool: SarifTool {
            driver: ripley_tool_component(rules),
        },
        results: sarif_results,
        invocations: Vec::new(),
    }])
}

pub fn behavioral_to_sarif(
    report: &BehavioralReport,
) -> (Vec<SarifReportingDescriptor>, Vec<SarifResult>) {
    let mut rules: Vec<SarifReportingDescriptor> = Vec::new();
    let mut rule_index_map: HashMap<String, usize> = HashMap::new();
    let mut results = Vec::new();

    for anomaly in &report.anomalies {
        let rule_id = format!("behavioral/{:?}", anomaly.kind).to_lowercase();

        let rule_idx = *rule_index_map.entry(rule_id.clone()).or_insert_with(|| {
            let idx = rules.len();
            rules.push(SarifReportingDescriptor {
                id: rule_id.clone(),
                name: format!("{:?}", anomaly.kind),
                short_description: SarifMessage {
                    text: format!("Behavioral anomaly: {:?}", anomaly.kind),
                },
                full_description: None,
                default_configuration: SarifConfiguration {
                    level: severity_to_sarif_level(&anomaly.severity),
                },
                help_uri: None,
            });
            idx
        });

        results.push(SarifResult {
            rule_id,
            rule_index: rule_idx,
            level: severity_to_sarif_level(&anomaly.severity),
            message: SarifMessage {
                text: anomaly.description.clone(),
            },
            locations: Vec::new(),
            fingerprints: HashMap::from([(
                "ripley/behavioral/v1".to_string(),
                format!("{}/{}/{}", report.package, report.version, anomaly.evidence),
            )]),
        });
    }

    (rules, results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::{HighlightedLine, MatchedRule};
    use crate::behavioral::report::{
        AnomalyKind, BehavioralAnomaly, DeclaredBehavior, ObservedBehavior,
    };
    use crate::feed::{Advisory, AffectedRange, FeedSource};
    use crate::lockfile::InstalledPackage;
    use crate::types::Ecosystem;

    #[test]
    fn test_sarif_log_serialization() {
        let log = new_sarif_log(vec![SarifRun {
            tool: SarifTool {
                driver: ripley_tool_component(Vec::new()),
            },
            results: Vec::new(),
            invocations: Vec::new(),
        }]);

        let json = serde_json::to_string_pretty(&log).expect("serialize");
        assert!(json.contains("\"$schema\""));
        assert!(json.contains(SARIF_SCHEMA));
        assert!(json.contains("\"version\": \"2.1.0\""));
    }

    #[test]
    fn test_severity_to_sarif_level() {
        assert_eq!(
            severity_to_sarif_level(&Severity::Critical),
            SarifLevel::Error
        );
        assert_eq!(severity_to_sarif_level(&Severity::High), SarifLevel::Error);
        assert_eq!(
            severity_to_sarif_level(&Severity::Medium),
            SarifLevel::Warning
        );
        assert_eq!(severity_to_sarif_level(&Severity::Low), SarifLevel::Note);
    }

    #[test]
    fn test_empty_results_valid_log() {
        let log = new_sarif_log(vec![SarifRun {
            tool: SarifTool {
                driver: ripley_tool_component(Vec::new()),
            },
            results: Vec::new(),
            invocations: Vec::new(),
        }]);

        let json = serde_json::to_string(&log).expect("serialize");
        let parsed: SarifLog = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.version, SARIF_VERSION);
        assert_eq!(parsed.runs.len(), 1);
        assert!(parsed.runs[0].results.is_empty());
    }

    #[test]
    fn test_sarif_result_roundtrip() {
        let result = SarifResult {
            rule_id: "GHSA-test-001".to_string(),
            rule_index: 0,
            level: SarifLevel::Error,
            message: SarifMessage {
                text: "Prototype pollution in lodash".to_string(),
            },
            locations: vec![SarifLocation {
                physical_location: SarifPhysicalLocation {
                    artifact_location: SarifArtifactLocation {
                        uri: "package-lock.json".to_string(),
                        uri_base_id: Some("%SRCROOT%".to_string()),
                    },
                    region: Some(SarifRegion {
                        start_line: 42,
                        start_column: None,
                    }),
                },
            }],
            fingerprints: HashMap::from([(
                "primaryLocationLineHash".to_string(),
                "abc123".to_string(),
            )]),
        };

        let json = serde_json::to_string(&result).expect("serialize");
        let parsed: SarifResult = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.rule_id, "GHSA-test-001");
        assert_eq!(parsed.level, SarifLevel::Error);
        assert_eq!(parsed.locations.len(), 1);
        assert_eq!(
            parsed.locations[0]
                .physical_location
                .region
                .as_ref()
                .map(|r| r.start_line),
            Some(42)
        );
    }

    #[test]
    fn test_sarif_level_serialization() {
        let json = serde_json::to_string(&SarifLevel::Error).expect("serialize");
        assert_eq!(json, "\"error\"");

        let json = serde_json::to_string(&SarifLevel::Warning).expect("serialize");
        assert_eq!(json, "\"warning\"");

        let json = serde_json::to_string(&SarifLevel::Note).expect("serialize");
        assert_eq!(json, "\"note\"");

        let json = serde_json::to_string(&SarifLevel::None).expect("serialize");
        assert_eq!(json, "\"none\"");
    }

    #[test]
    fn test_reporting_descriptor_serialization() {
        let desc = SarifReportingDescriptor {
            id: "GHSA-test-001".to_string(),
            name: "prototype-pollution".to_string(),
            short_description: SarifMessage {
                text: "Prototype Pollution in lodash".to_string(),
            },
            full_description: None,
            default_configuration: SarifConfiguration {
                level: SarifLevel::Error,
            },
            help_uri: Some("https://github.com/advisories/GHSA-test-001".to_string()),
        };

        let json = serde_json::to_string_pretty(&desc).expect("serialize");
        assert!(json.contains("\"shortDescription\""));
        assert!(json.contains("\"defaultConfiguration\""));
        assert!(json.contains("\"helpUri\""));
        assert!(!json.contains("\"fullDescription\""));
    }

    fn test_advisory(id: &str, package: &str, severity: Severity) -> Advisory {
        Advisory {
            id: id.to_string(),
            source: FeedSource::Osv,
            ecosystem: Ecosystem::Npm,
            package: package.to_string(),
            affected_ranges: vec![AffectedRange {
                introduced: semver::Version::new(0, 0, 0),
                fixed: Some(semver::Version::new(2, 0, 0)),
            }],
            severity: Some(severity),
            summary: format!("Vulnerability in {package}"),
            references: vec![format!("https://github.com/advisories/{id}")],
        }
    }

    fn test_match(id: &str, package: &str, version: &str, severity: Severity) -> Match {
        Match {
            advisory: test_advisory(id, package, severity),
            package: InstalledPackage {
                name: package.to_string(),
                version: semver::Version::parse(version).expect("parse"),
                ecosystem: Ecosystem::Npm,
            },
            project_path: PathBuf::from("/project"),
        }
    }

    #[test]
    fn test_single_match_to_sarif() {
        let matches = vec![test_match("GHSA-001", "lodash", "4.17.20", Severity::High)];
        let log = matches_to_sarif(&matches, &[], &[]);

        assert_eq!(log.runs.len(), 1);
        assert_eq!(log.runs[0].results.len(), 1);
        assert_eq!(log.runs[0].tool.driver.rules.len(), 1);

        let result = &log.runs[0].results[0];
        assert_eq!(result.rule_id, "GHSA-001");
        assert_eq!(result.level, SarifLevel::Error);
        assert!(result.message.text.contains("lodash@4.17.20"));
    }

    #[test]
    fn test_multiple_matches_deduped_rules() {
        let matches = vec![
            test_match("GHSA-001", "lodash", "4.17.20", Severity::High),
            test_match("GHSA-001", "lodash", "4.17.19", Severity::High),
            test_match("GHSA-002", "express", "4.17.0", Severity::Critical),
        ];
        let log = matches_to_sarif(&matches, &[], &[]);

        assert_eq!(log.runs[0].results.len(), 3);
        assert_eq!(log.runs[0].tool.driver.rules.len(), 2);
    }

    #[test]
    fn test_warnings_mapped_to_sarif() {
        let warnings = vec![LockfileWarning {
            package: "test-pkg".to_string(),
            field: "integrity".to_string(),
            message: "missing integrity hash".to_string(),
            severity: Severity::Medium,
        }];
        let log = matches_to_sarif(&[], &warnings, &[]);

        assert_eq!(log.runs[0].results.len(), 1);
        let result = &log.runs[0].results[0];
        assert_eq!(result.level, SarifLevel::Warning);
        assert!(result.message.text.contains("test-pkg"));
    }

    #[test]
    fn test_risky_specs_mapped_to_sarif() {
        let risky = vec![RiskySpec {
            package: "evil-pkg".to_string(),
            specifier: "git+https://github.com/evil/pkg".to_string(),
            reason: "git dependency".to_string(),
        }];
        let log = matches_to_sarif(&[], &[], &risky);

        assert_eq!(log.runs[0].results.len(), 1);
        let result = &log.runs[0].results[0];
        assert_eq!(result.rule_id, "ripley/posture/risky-spec");
        assert_eq!(result.level, SarifLevel::Warning);
    }

    #[test]
    fn test_sarif_fingerprint_deterministic() {
        let fp1 = sarif_fingerprint("GHSA-001", "lodash", "4.17.20");
        let fp2 = sarif_fingerprint("GHSA-001", "lodash", "4.17.20");
        assert_eq!(fp1, fp2);

        let fp3 = sarif_fingerprint("GHSA-001", "lodash", "4.17.19");
        assert_ne!(fp1, fp3);
    }

    #[test]
    fn test_analysis_to_sarif() {
        let results = vec![AnalysisResult {
            risk_level: Severity::High,
            matched_rules: vec![MatchedRule {
                rule_id: "npm-network-call".to_string(),
                rule_name: "Network call in postinstall".to_string(),
                matched_line: 5,
                matched_text: "curl https://evil.com".to_string(),
            }],
            highlighted_lines: vec![HighlightedLine {
                line_number: 5,
                text: "curl https://evil.com".to_string(),
                severity: Severity::High,
            }],
        }];
        let paths = vec![PathBuf::from("scripts/postinstall.sh")];
        let log = analysis_to_sarif(&results, &paths);

        assert_eq!(log.runs[0].results.len(), 1);
        let result = &log.runs[0].results[0];
        assert_eq!(result.rule_id, "npm-network-call");
        assert_eq!(result.level, SarifLevel::Error);
        assert_eq!(
            result.locations[0]
                .physical_location
                .region
                .as_ref()
                .map(|r| r.start_line),
            Some(5)
        );
    }

    #[test]
    fn test_behavioral_to_sarif() {
        let report = BehavioralReport {
            package: "evil-pkg".to_string(),
            version: "1.0.0".to_string(),
            ecosystem: Ecosystem::Npm,
            declared: DeclaredBehavior::default(),
            observed: ObservedBehavior::default(),
            anomalies: vec![
                BehavioralAnomaly {
                    kind: AnomalyKind::UnexpectedNetwork,
                    severity: Severity::High,
                    description: "network access to evil.com".to_string(),
                    evidence: "evil.com".to_string(),
                },
                BehavioralAnomaly {
                    kind: AnomalyKind::CredentialAccess,
                    severity: Severity::Critical,
                    description: "read ~/.ssh/id_rsa".to_string(),
                    evidence: "~/.ssh/id_rsa".to_string(),
                },
            ],
            risk_score: 0.65,
            analysis_duration_ms: 50,
        };

        let (rules, results) = behavioral_to_sarif(&report);
        assert_eq!(results.len(), 2);
        assert_eq!(rules.len(), 2);
        assert_eq!(results[0].level, SarifLevel::Error);
        assert_eq!(results[1].level, SarifLevel::Error);
        assert_eq!(results[0].rule_index, 0);
        assert_eq!(results[1].rule_index, 1);
        assert!(results[0].rule_id.contains("unexpectednetwork"));
        assert!(results[1].rule_id.contains("credentialaccess"));
        assert!(results[0].fingerprints.contains_key("ripley/behavioral/v1"));
        assert_eq!(rules[0].id, results[0].rule_id);
        assert_eq!(rules[1].id, results[1].rule_id);
    }
}
