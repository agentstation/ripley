use std::path::{Path, PathBuf};

use serde::Deserialize;
use tracing;

use crate::types::{Ecosystem, Severity};

#[derive(Debug, thiserror::Error)]
pub enum RuleError {
    #[error("could not parse compiled rules: {0}")]
    ParseCompiled(toml::de::Error),
    #[error("could not parse rule file {path}: {source}")]
    ParseFile {
        path: PathBuf,
        source: toml::de::Error,
    },
    #[error("could not read rule directory {path}: {source}")]
    ReadDir {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("could not read rule file {path}: {source}")]
    ReadFile {
        path: PathBuf,
        source: std::io::Error,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub ecosystem: String,
    pub signal: String,
    pub weight: Severity,
    pub patterns: Vec<String>,
}

#[derive(Deserialize)]
struct RuleFile {
    rules: Vec<Rule>,
}

#[derive(Debug)]
pub struct RuleSet {
    rules: Vec<Rule>,
}

impl RuleSet {
    pub fn load_compiled() -> Result<Self, RuleError> {
        let mut rules = Vec::new();

        let sources: &[&str] = &[
            include_str!("../../../../rules/npm_postinstall.toml"),
            include_str!("../../../../rules/credential_exfil.toml"),
            include_str!("../../../../rules/persistence_write.toml"),
        ];

        for source in sources {
            let file: RuleFile = toml::from_str(source).map_err(RuleError::ParseCompiled)?;
            rules.extend(file.rules);
        }

        Ok(Self { rules })
    }

    pub fn load_user_rules(dir: &Path) -> Result<Self, RuleError> {
        let mut rules = Vec::new();

        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self { rules });
            }
            Err(e) => {
                return Err(RuleError::ReadDir {
                    path: dir.to_path_buf(),
                    source: e,
                });
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    tracing::warn!("could not read entry in {}: {e}", dir.display());
                    continue;
                }
            };
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("toml") {
                continue;
            }

            let content = std::fs::read_to_string(&path).map_err(|e| RuleError::ReadFile {
                path: path.clone(),
                source: e,
            })?;

            let parsed = parse_rule_content(&content).map_err(|e| RuleError::ParseFile {
                path: path.clone(),
                source: e,
            })?;
            rules.extend(parsed);
        }

        Ok(Self { rules })
    }

    pub fn merge(base: Self, user: Self) -> Self {
        let mut rules = base.rules;
        for user_rule in user.rules {
            if let Some(pos) = rules.iter().position(|r| r.id == user_rule.id) {
                rules[pos] = user_rule;
            } else {
                rules.push(user_rule);
            }
        }
        Self { rules }
    }

    pub fn for_ecosystem(&self, eco: Ecosystem) -> Vec<&Rule> {
        self.rules
            .iter()
            .filter(|r| ecosystem_matches(&r.ecosystem, eco))
            .collect()
    }

    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }
}

fn parse_rule_content(content: &str) -> Result<Vec<Rule>, toml::de::Error> {
    match toml::from_str::<RuleFile>(content) {
        Ok(file) => return Ok(file.rules),
        Err(e) => tracing::debug!("not a [[rules]] array format, trying single-rule: {e}"),
    }
    let rule: Rule = toml::from_str(content)?;
    Ok(vec![rule])
}

fn ecosystem_matches(rule_eco: &str, eco: Ecosystem) -> bool {
    match rule_eco {
        "all" => true,
        "npm" => eco == Ecosystem::Npm,
        "pypi" => eco == Ecosystem::PyPI,
        "cargo" => eco == Ecosystem::Cargo,
        "go" => eco == Ecosystem::Go,
        "gem" => eco == Ecosystem::Gem,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_compiled_rules() {
        let ruleset = RuleSet::load_compiled().expect("load compiled rules");
        let rules = ruleset.rules();
        assert!(!rules.is_empty(), "should have compiled rules");

        let network_rule = rules
            .iter()
            .find(|r| r.id == "NPM001")
            .expect("should have NPM001 rule");
        assert_eq!(network_rule.ecosystem, "npm");
        assert_eq!(network_rule.signal, "network_call");
        assert_eq!(network_rule.weight, Severity::High);
        assert!(!network_rule.patterns.is_empty());
    }

    #[test]
    fn test_user_override() {
        let base = RuleSet::load_compiled().expect("load compiled rules");

        let dir = tempfile::tempdir().expect("create tempdir");
        let override_path = dir.path().join("override.toml");
        std::fs::write(
            &override_path,
            r#"
id = "NPM001"
name = "overridden-rule"
description = "User override"
ecosystem = "npm"
signal = "network_call"
weight = "low"
patterns = []
"#,
        )
        .expect("write override");

        let user = RuleSet::load_user_rules(dir.path()).expect("load user rules");
        let merged = RuleSet::merge(base, user);

        let rule = merged
            .rules()
            .iter()
            .find(|r| r.id == "NPM001")
            .expect("find rule");
        assert_eq!(rule.name, "overridden-rule");
        assert_eq!(rule.weight, Severity::Low);
        assert!(rule.patterns.is_empty());
    }

    #[test]
    fn test_for_ecosystem_filters() {
        let ruleset = RuleSet::load_compiled().expect("load compiled rules");
        let npm_rules = ruleset.for_ecosystem(Ecosystem::Npm);
        assert!(!npm_rules.is_empty());

        for rule in &npm_rules {
            assert!(
                rule.ecosystem == "npm" || rule.ecosystem == "all",
                "rule {} has ecosystem {}",
                rule.id,
                rule.ecosystem
            );
        }
    }

    #[test]
    fn test_user_rules_nonexistent_dir() {
        let ruleset =
            RuleSet::load_user_rules(Path::new("/nonexistent/path")).expect("should succeed");
        assert!(ruleset.rules().is_empty());
    }

    #[test]
    fn test_merge_appends_new_rules() {
        let base = RuleSet {
            rules: vec![Rule {
                id: "BASE001".to_string(),
                name: "base".to_string(),
                description: "base rule".to_string(),
                ecosystem: "npm".to_string(),
                signal: "test".to_string(),
                weight: Severity::Low,
                patterns: vec![],
            }],
        };
        let user = RuleSet {
            rules: vec![Rule {
                id: "USER001".to_string(),
                name: "user".to_string(),
                description: "user rule".to_string(),
                ecosystem: "npm".to_string(),
                signal: "test".to_string(),
                weight: Severity::High,
                patterns: vec![],
            }],
        };

        let merged = RuleSet::merge(base, user);
        assert_eq!(merged.rules().len(), 2);
        assert!(merged.rules().iter().any(|r| r.id == "BASE001"));
        assert!(merged.rules().iter().any(|r| r.id == "USER001"));
    }
}
