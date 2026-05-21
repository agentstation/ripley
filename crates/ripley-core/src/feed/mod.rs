pub mod ghsa;
pub mod osv;
pub mod socket;

use serde::{Deserialize, Serialize};

use crate::types::{Ecosystem, Severity};

#[derive(Debug, thiserror::Error)]
pub enum FeedError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("failed to parse feed response: {0}")]
    Parse(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum FeedSource {
    #[default]
    Osv,
    Ghsa,
    Socket,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Advisory {
    pub id: String,
    #[serde(default)]
    pub source: FeedSource,
    pub ecosystem: Ecosystem,
    pub package: String,
    pub affected_ranges: Vec<AffectedRange>,
    pub severity: Option<Severity>,
    pub summary: String,
    pub references: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectedRange {
    pub introduced: semver::Version,
    pub fixed: Option<semver::Version>,
}

pub fn deduplicate_advisories(advisories: Vec<Advisory>) -> Vec<Advisory> {
    let mut seen: std::collections::HashMap<String, Advisory> = std::collections::HashMap::new();

    for adv in advisories {
        match seen.get(&adv.id) {
            Some(existing)
                if existing.ecosystem == adv.ecosystem && existing.package == adv.package =>
            {
                let mut merged = existing.clone();
                if merged.severity.is_none() && adv.severity.is_some() {
                    merged.severity = adv.severity;
                }
                if merged.summary.is_empty() && !adv.summary.is_empty() {
                    merged.summary = adv.summary.clone();
                }
                for r in &adv.references {
                    if !merged.references.contains(r) {
                        merged.references.push(r.clone());
                    }
                }
                for range in &adv.affected_ranges {
                    if !merged
                        .affected_ranges
                        .iter()
                        .any(|er| er.introduced == range.introduced && er.fixed == range.fixed)
                    {
                        merged.affected_ranges.push(range.clone());
                    }
                }
                seen.insert(adv.id, merged);
            }
            _ => {
                seen.insert(adv.id.clone(), adv);
            }
        }
    }

    let mut result: Vec<Advisory> = seen.into_values().collect();
    result.sort_by(|a, b| a.id.cmp(&b.id));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate_merges_same_id() {
        let adv1 = Advisory {
            id: "CVE-2024-001".to_string(),
            source: FeedSource::Osv,
            ecosystem: Ecosystem::Npm,
            package: "lodash".to_string(),
            affected_ranges: vec![AffectedRange {
                introduced: semver::Version::new(1, 0, 0),
                fixed: Some(semver::Version::new(1, 2, 0)),
            }],
            severity: None,
            summary: "".to_string(),
            references: vec!["https://osv.dev/CVE-2024-001".to_string()],
        };
        let adv2 = Advisory {
            id: "CVE-2024-001".to_string(),
            source: FeedSource::Ghsa,
            ecosystem: Ecosystem::Npm,
            package: "lodash".to_string(),
            affected_ranges: vec![],
            severity: Some(Severity::High),
            summary: "Prototype pollution in lodash".to_string(),
            references: vec!["https://github.com/advisories/GHSA-xxx".to_string()],
        };

        let result = deduplicate_advisories(vec![adv1, adv2]);
        assert_eq!(result.len(), 1);
        let merged = &result[0];
        assert_eq!(merged.severity, Some(Severity::High));
        assert_eq!(merged.summary, "Prototype pollution in lodash");
        assert_eq!(merged.references.len(), 2);
        assert_eq!(merged.affected_ranges.len(), 1);
    }

    #[test]
    fn test_deduplicate_keeps_distinct() {
        let adv1 = Advisory {
            id: "CVE-2024-001".to_string(),
            source: FeedSource::Osv,
            ecosystem: Ecosystem::Npm,
            package: "lodash".to_string(),
            affected_ranges: vec![],
            severity: Some(Severity::High),
            summary: "first".to_string(),
            references: vec![],
        };
        let adv2 = Advisory {
            id: "CVE-2024-002".to_string(),
            source: FeedSource::Ghsa,
            ecosystem: Ecosystem::Npm,
            package: "express".to_string(),
            affected_ranges: vec![],
            severity: Some(Severity::Medium),
            summary: "second".to_string(),
            references: vec![],
        };

        let result = deduplicate_advisories(vec![adv1, adv2]);
        assert_eq!(result.len(), 2);
    }
}
