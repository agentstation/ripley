pub mod fetcher;
pub mod registry;

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub ecosystem: String,
    pub signal: String,
    pub weight: Severity,
    pub patterns: Vec<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub confidence: Option<u8>,
    #[serde(default)]
    pub source_attack: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub min_ripley_version: Option<String>,
    #[serde(default)]
    pub source: RuleSource,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RuleSource {
    #[default]
    Compiled,
    User,
    Community {
        source_name: String,
    },
}

impl Rule {
    pub fn is_community(&self) -> bool {
        matches!(self.source, RuleSource::Community { .. })
    }

    pub fn is_trusted(&self, trusted_sources: &[String]) -> bool {
        match &self.source {
            RuleSource::Compiled | RuleSource::User => true,
            RuleSource::Community { source_name } => trusted_sources.contains(source_name),
        }
    }
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
            include_str!("../../../../rules/pypi_setup.toml"),
            include_str!("../../../../rules/cargo_build.toml"),
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

    pub fn load_community_rules(data_dir: &Path) -> Result<Self, RuleError> {
        let community_dir = data_dir.join("rules").join("community");
        let mut rules = Vec::new();

        let sources = match std::fs::read_dir(&community_dir) {
            Ok(e) => e,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self { rules });
            }
            Err(e) => {
                return Err(RuleError::ReadDir {
                    path: community_dir,
                    source: e,
                });
            }
        };

        for source_entry in sources {
            let source_entry = match source_entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            let source_path = source_entry.path();
            if !source_path.is_dir() {
                continue;
            }
            let source_name = source_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            let entries = match std::fs::read_dir(&source_path) {
                Ok(e) => e,
                Err(_) => continue,
            };

            for entry in entries {
                let entry = match entry {
                    Ok(e) => e,
                    Err(_) => continue,
                };
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("toml") {
                    continue;
                }

                let content = match std::fs::read_to_string(&path) {
                    Ok(c) => c,
                    Err(e) => {
                        tracing::warn!("could not read community rule {}: {e}", path.display());
                        continue;
                    }
                };

                match parse_rule_content(&content) {
                    Ok(mut parsed_rules) => {
                        for rule in &mut parsed_rules {
                            rule.source = RuleSource::Community {
                                source_name: source_name.clone(),
                            };
                        }
                        rules.extend(parsed_rules);
                    }
                    Err(e) => {
                        tracing::warn!("could not parse community rule {}: {e}", path.display());
                    }
                }
            }
        }

        Ok(Self { rules })
    }

    pub fn load_all(config_dir: &Path, data_dir: &Path) -> Result<Self, RuleError> {
        let mut compiled = Self::load_compiled()?;
        for rule in &mut compiled.rules {
            rule.source = RuleSource::Compiled;
        }

        let community = Self::load_community_rules(data_dir)?;

        let user_rules_dir = config_dir.join("rules");
        let mut user = Self::load_user_rules(&user_rules_dir)?;
        for rule in &mut user.rules {
            rule.source = RuleSource::User;
        }

        // Three-tier merge: compiled < community < user
        let merged = Self::merge(compiled, community);
        Ok(Self::merge(merged, user))
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
    fn test_pypi_rules_load() {
        let ruleset = RuleSet::load_compiled().expect("load compiled rules");
        let pypi_rules = ruleset.for_ecosystem(Ecosystem::PyPI);
        assert!(
            pypi_rules.len() >= 8,
            "expected at least 8 PyPI rules, got {}",
            pypi_rules.len()
        );

        let os_system = pypi_rules
            .iter()
            .find(|r| r.id == "PYPI001")
            .expect("PYPI001 should exist");
        assert_eq!(os_system.signal, "shell_spawning");
        assert_eq!(os_system.weight, Severity::High);

        for rule in &pypi_rules {
            assert!(
                rule.ecosystem == "pypi" || rule.ecosystem == "all",
                "rule {} has ecosystem {}",
                rule.id,
                rule.ecosystem
            );
        }
    }

    #[test]
    fn test_cargo_rules_load() {
        let ruleset = RuleSet::load_compiled().expect("load compiled rules");
        let cargo_rules = ruleset.for_ecosystem(Ecosystem::Cargo);
        assert!(
            cargo_rules.len() >= 7,
            "expected at least 7 Cargo rules, got {}",
            cargo_rules.len()
        );

        let cmd_exec = cargo_rules
            .iter()
            .find(|r| r.id == "CARGO001")
            .expect("CARGO001 should exist");
        assert_eq!(cmd_exec.signal, "shell_spawning");
        assert_eq!(cmd_exec.weight, Severity::High);

        for rule in &cargo_rules {
            assert!(
                rule.ecosystem == "cargo" || rule.ecosystem == "all",
                "rule {} has ecosystem {}",
                rule.id,
                rule.ecosystem
            );
        }
    }

    #[test]
    fn test_merge_appends_new_rules() {
        let base = RuleSet {
            rules: vec![test_rule("BASE001", "base", Severity::Low)],
        };
        let user = RuleSet {
            rules: vec![test_rule("USER001", "user", Severity::High)],
        };

        let merged = RuleSet::merge(base, user);
        assert_eq!(merged.rules().len(), 2);
        assert!(merged.rules().iter().any(|r| r.id == "BASE001"));
        assert!(merged.rules().iter().any(|r| r.id == "USER001"));
    }

    #[test]
    fn test_rule_existing_parse_unchanged() {
        let rules = RuleSet::load_compiled().expect("load");
        for rule in rules.rules() {
            assert!(!rule.id.is_empty());
            assert_eq!(rule.source, RuleSource::Compiled);
        }
    }

    #[test]
    fn test_rule_full_metadata_parses() {
        let content = r#"
[[rules]]
id = "FULL001"
name = "full rule"
description = "a fully specified rule"
ecosystem = "npm"
signal = "network"
weight = "high"
patterns = ["curl\\s+"]
author = "test-author"
confidence = 90
source_attack = "exfiltration"
updated_at = "2026-01-01"
min_ripley_version = "0.5.0"
"#;
        let parsed = parse_rule_content(content).expect("parse");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].author.as_deref(), Some("test-author"));
        assert_eq!(parsed[0].confidence, Some(90));
    }

    #[test]
    fn test_is_community() {
        let mut rule = test_rule("TEST001", "test", Severity::Low);
        assert!(!rule.is_community());

        rule.source = RuleSource::Community {
            source_name: "test-src".to_string(),
        };
        assert!(rule.is_community());
    }

    #[test]
    fn test_is_trusted() {
        let mut rule = test_rule("TEST001", "test", Severity::Low);
        assert!(rule.is_trusted(&[]));

        rule.source = RuleSource::User;
        assert!(rule.is_trusted(&[]));

        rule.source = RuleSource::Community {
            source_name: "my-src".to_string(),
        };
        assert!(!rule.is_trusted(&[]));
        assert!(rule.is_trusted(&["my-src".to_string()]));
        assert!(!rule.is_trusted(&["other-src".to_string()]));
    }

    #[test]
    fn test_load_community_rules_empty() {
        let tmpdir = tempfile::tempdir().expect("tmpdir");
        let rules = RuleSet::load_community_rules(tmpdir.path()).expect("load");
        assert!(rules.rules().is_empty());
    }

    #[test]
    fn test_load_all_three_tier() {
        let tmpdir = tempfile::tempdir().expect("tmpdir");
        let config_dir = tmpdir.path().join("config");
        let data_dir = tmpdir.path().join("data");
        std::fs::create_dir_all(config_dir.join("rules")).expect("mkdir");

        let rules = RuleSet::load_all(&config_dir, &data_dir).expect("load");
        assert!(!rules.rules().is_empty());
        assert!(
            rules
                .rules()
                .iter()
                .all(|r| r.source == RuleSource::Compiled)
        );
    }

    fn test_rule(id: &str, name: &str, weight: Severity) -> Rule {
        Rule {
            id: id.to_string(),
            name: name.to_string(),
            description: format!("{name} rule"),
            ecosystem: "npm".to_string(),
            signal: "test".to_string(),
            weight,
            patterns: vec![],
            author: None,
            confidence: None,
            source_attack: None,
            updated_at: None,
            min_ripley_version: None,
            source: RuleSource::Compiled,
        }
    }
}
