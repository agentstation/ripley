use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("source '{0}' already exists")]
    DuplicateSource(String),
    #[error("source '{0}' not found")]
    SourceNotFound(String),
    #[error("could not read registry at {path}: {source}")]
    ReadFile {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("could not parse registry: {0}")]
    ParseError(#[from] toml::de::Error),
    #[error("could not write registry at {path}: {source}")]
    WriteFile {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("could not serialize registry: {0}")]
    SerializeError(#[from] toml::ser::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSourceEntry {
    pub name: String,
    pub url: String,
    pub trust_level: TrustLevel,
    pub last_fetched: Option<String>,
    pub rule_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustLevel {
    Untrusted,
    Trusted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleIndex {
    pub version: u32,
    pub rules: Vec<RuleEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleEntry {
    pub file: String,
    pub id: String,
    pub name: String,
    pub ecosystem: String,
    pub weight: String,
    pub author: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SourceRegistry {
    #[serde(default)]
    pub sources: Vec<RuleSourceEntry>,
}

impl SourceRegistry {
    pub fn load(data_dir: &Path) -> Result<Self, RegistryError> {
        let path = Self::registry_path(data_dir);
        match std::fs::read_to_string(&path) {
            Ok(content) => {
                let registry: Self = toml::from_str(&content)?;
                Ok(registry)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(RegistryError::ReadFile { path, source: e }),
        }
    }

    pub fn save(&self, data_dir: &Path) -> Result<(), RegistryError> {
        let path = Self::registry_path(data_dir);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| RegistryError::WriteFile {
                path: path.clone(),
                source: e,
            })?;
        }
        let content = toml::to_string_pretty(self)?;
        let tmp_path = path.with_extension("toml.tmp");
        std::fs::write(&tmp_path, &content).map_err(|e| RegistryError::WriteFile {
            path: path.clone(),
            source: e,
        })?;
        std::fs::rename(&tmp_path, &path)
            .map_err(|e| RegistryError::WriteFile { path, source: e })?;
        Ok(())
    }

    pub fn add_source(&mut self, name: String, url: String) -> Result<(), RegistryError> {
        if self.sources.iter().any(|s| s.name == name) {
            return Err(RegistryError::DuplicateSource(name));
        }
        self.sources.push(RuleSourceEntry {
            name,
            url,
            trust_level: TrustLevel::Untrusted,
            last_fetched: None,
            rule_count: 0,
        });
        Ok(())
    }

    pub fn remove_source(&mut self, name: &str) -> Result<(), RegistryError> {
        let pos = self
            .sources
            .iter()
            .position(|s| s.name == name)
            .ok_or_else(|| RegistryError::SourceNotFound(name.to_string()))?;
        self.sources.remove(pos);
        Ok(())
    }

    pub fn trust_source(&mut self, name: &str) -> Result<(), RegistryError> {
        let source = self
            .sources
            .iter_mut()
            .find(|s| s.name == name)
            .ok_or_else(|| RegistryError::SourceNotFound(name.to_string()))?;
        source.trust_level = TrustLevel::Trusted;
        Ok(())
    }

    pub fn untrust_source(&mut self, name: &str) -> Result<(), RegistryError> {
        let source = self
            .sources
            .iter_mut()
            .find(|s| s.name == name)
            .ok_or_else(|| RegistryError::SourceNotFound(name.to_string()))?;
        source.trust_level = TrustLevel::Untrusted;
        Ok(())
    }

    pub fn trusted_source_names(&self) -> Vec<String> {
        self.sources
            .iter()
            .filter(|s| s.trust_level == TrustLevel::Trusted)
            .map(|s| s.name.clone())
            .collect()
    }

    fn registry_path(data_dir: &Path) -> PathBuf {
        data_dir.join("rules").join("sources.toml")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_source() {
        let mut registry = SourceRegistry::default();
        registry
            .add_source(
                "community-rules".to_string(),
                "https://example.com/rules".to_string(),
            )
            .expect("add source");
        assert_eq!(registry.sources.len(), 1);
        assert_eq!(registry.sources[0].name, "community-rules");
        assert_eq!(registry.sources[0].trust_level, TrustLevel::Untrusted);
    }

    #[test]
    fn test_duplicate_source_error() {
        let mut registry = SourceRegistry::default();
        registry
            .add_source("dup".to_string(), "https://a.com".to_string())
            .expect("first add");
        let result = registry.add_source("dup".to_string(), "https://b.com".to_string());
        assert!(matches!(result, Err(RegistryError::DuplicateSource(_))));
    }

    #[test]
    fn test_remove_source() {
        let mut registry = SourceRegistry::default();
        registry
            .add_source("test".to_string(), "https://test.com".to_string())
            .expect("add");
        registry.remove_source("test").expect("remove");
        assert!(registry.sources.is_empty());
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut registry = SourceRegistry::default();
        let result = registry.remove_source("nope");
        assert!(matches!(result, Err(RegistryError::SourceNotFound(_))));
    }

    #[test]
    fn test_trust_untrust() {
        let mut registry = SourceRegistry::default();
        registry
            .add_source("src".to_string(), "https://src.com".to_string())
            .expect("add");
        assert_eq!(registry.sources[0].trust_level, TrustLevel::Untrusted);

        registry.trust_source("src").expect("trust");
        assert_eq!(registry.sources[0].trust_level, TrustLevel::Trusted);

        registry.untrust_source("src").expect("untrust");
        assert_eq!(registry.sources[0].trust_level, TrustLevel::Untrusted);
    }

    #[test]
    fn test_trusted_source_names() {
        let mut registry = SourceRegistry::default();
        registry
            .add_source("trusted-one".to_string(), "https://a.com".to_string())
            .expect("add");
        registry
            .add_source("untrusted-one".to_string(), "https://b.com".to_string())
            .expect("add");
        registry.trust_source("trusted-one").expect("trust");

        let trusted = registry.trusted_source_names();
        assert_eq!(trusted, vec!["trusted-one".to_string()]);
    }

    #[test]
    fn test_roundtrip_persistence() {
        let tmpdir = tempfile::tempdir().expect("tmpdir");
        let mut registry = SourceRegistry::default();
        registry
            .add_source("test-src".to_string(), "https://test.com/rules".to_string())
            .expect("add");
        registry.trust_source("test-src").expect("trust");

        registry.save(tmpdir.path()).expect("save");
        let loaded = SourceRegistry::load(tmpdir.path()).expect("load");
        assert_eq!(loaded.sources.len(), 1);
        assert_eq!(loaded.sources[0].name, "test-src");
        assert_eq!(loaded.sources[0].trust_level, TrustLevel::Trusted);
    }

    #[test]
    fn test_empty_registry_loads() {
        let tmpdir = tempfile::tempdir().expect("tmpdir");
        let loaded = SourceRegistry::load(tmpdir.path()).expect("load");
        assert!(loaded.sources.is_empty());
    }

    #[test]
    fn test_rule_index_deserialization() {
        let toml_str = r#"
version = 1

[[rules]]
file = "supply_chain_2026.toml"
id = "SC2026-001"
name = "npm postinstall exfiltration"
ecosystem = "npm"
weight = "high"
author = "community"
"#;
        let index: RuleIndex = toml::from_str(toml_str).expect("parse");
        assert_eq!(index.version, 1);
        assert_eq!(index.rules.len(), 1);
        assert_eq!(index.rules[0].id, "SC2026-001");
    }
}
