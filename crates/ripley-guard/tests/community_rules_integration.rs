use std::path::PathBuf;

use ripley_core::rules::registry::{RuleIndex, SourceRegistry, TrustLevel};
use ripley_core::rules::{RuleSet, RuleSource};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("fixtures")
}

fn example_rules_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("docs")
        .join("community-rules-example")
}

#[test]
fn test_example_index_parses() {
    let content =
        std::fs::read_to_string(example_rules_dir().join("index.toml")).expect("read index");
    let index: RuleIndex = toml::from_str(&content).expect("parse index");
    assert_eq!(index.version, 1);
    assert!(!index.rules.is_empty());
}

#[test]
fn test_example_rules_parse() {
    let content = std::fs::read_to_string(example_rules_dir().join("supply_chain_2026.toml"))
        .expect("read rules");
    let rules = ripley_core::rules::RuleSet::load_compiled().expect("compiled");
    let _ = rules;

    #[derive(serde::Deserialize)]
    struct RuleFile {
        rules: Vec<ripley_core::rules::Rule>,
    }
    let parsed: RuleFile = toml::from_str(&content).expect("parse rules");
    assert!(!parsed.rules.is_empty());
    assert_eq!(parsed.rules[0].id, "SC2026-001");
    assert_eq!(parsed.rules[0].author.as_deref(), Some("ripley-community"));
}

#[test]
fn test_community_rule_loads_from_directory() {
    let tmpdir = tempfile::tempdir().expect("tmpdir");
    let data_dir = tmpdir.path().join("data");
    let community_dir = data_dir.join("rules").join("community").join("test-src");
    std::fs::create_dir_all(&community_dir).expect("mkdir");

    std::fs::write(
        community_dir.join("test_rule.toml"),
        r#"
id = "COMM001"
name = "community test rule"
description = "a test community rule"
ecosystem = "npm"
signal = "test"
weight = "medium"
patterns = ["test_pattern"]
"#,
    )
    .expect("write");

    let rules = RuleSet::load_community_rules(&data_dir).expect("load");
    assert_eq!(rules.rules().len(), 1);
    assert_eq!(rules.rules()[0].id, "COMM001");
    assert!(rules.rules()[0].is_community());
}

#[test]
fn test_community_rule_untrusted_does_not_block() {
    let rule = ripley_core::rules::Rule {
        id: "COMM001".to_string(),
        name: "test".to_string(),
        description: "test".to_string(),
        ecosystem: "npm".to_string(),
        signal: "test".to_string(),
        weight: ripley_core::types::Severity::High,
        patterns: vec![],
        author: None,
        confidence: None,
        source_attack: None,
        updated_at: None,
        min_ripley_version: None,
        source: RuleSource::Community {
            source_name: "untrusted-src".to_string(),
        },
    };
    assert!(!rule.is_trusted(&[]));
    assert!(!rule.is_trusted(&["other-src".to_string()]));
}

#[test]
fn test_community_rule_trusted_blocks() {
    let rule = ripley_core::rules::Rule {
        id: "COMM001".to_string(),
        name: "test".to_string(),
        description: "test".to_string(),
        ecosystem: "npm".to_string(),
        signal: "test".to_string(),
        weight: ripley_core::types::Severity::High,
        patterns: vec![],
        author: None,
        confidence: None,
        source_attack: None,
        updated_at: None,
        min_ripley_version: None,
        source: RuleSource::Community {
            source_name: "trusted-src".to_string(),
        },
    };
    assert!(rule.is_trusted(&["trusted-src".to_string()]));
}

#[test]
fn test_three_tier_merge_precedence() {
    let tmpdir = tempfile::tempdir().expect("tmpdir");
    let config_dir = tmpdir.path().join("config");
    let data_dir = tmpdir.path().join("data");

    std::fs::create_dir_all(config_dir.join("rules")).expect("mkdir config");
    let community_dir = data_dir.join("rules").join("community").join("test-src");
    std::fs::create_dir_all(&community_dir).expect("mkdir community");

    // Community rule that overrides compiled NPM001
    std::fs::write(
        community_dir.join("override.toml"),
        r#"
id = "NPM001"
name = "community-override"
description = "community override of NPM001"
ecosystem = "npm"
signal = "network_call"
weight = "medium"
patterns = ["curl"]
"#,
    )
    .expect("write community");

    // User rule that overrides the same
    std::fs::write(
        config_dir.join("rules").join("override.toml"),
        r#"
id = "NPM001"
name = "user-override"
description = "user override of NPM001"
ecosystem = "npm"
signal = "network_call"
weight = "low"
patterns = []
"#,
    )
    .expect("write user");

    let rules = RuleSet::load_all(&config_dir, &data_dir).expect("load");
    let npm001 = rules
        .rules()
        .iter()
        .find(|r| r.id == "NPM001")
        .expect("find");
    // User takes precedence over community which takes precedence over compiled
    assert_eq!(npm001.name, "user-override");
    assert_eq!(npm001.weight, ripley_core::types::Severity::Low);
}

#[test]
fn test_registry_roundtrip() {
    let tmpdir = tempfile::tempdir().expect("tmpdir");
    let mut registry = SourceRegistry::default();
    registry
        .add_source(
            "test-rules".to_string(),
            "https://example.com/rules".to_string(),
        )
        .expect("add");
    registry.trust_source("test-rules").expect("trust");
    registry.save(tmpdir.path()).expect("save");

    let loaded = SourceRegistry::load(tmpdir.path()).expect("load");
    assert_eq!(loaded.sources.len(), 1);
    assert_eq!(loaded.sources[0].trust_level, TrustLevel::Trusted);
}
