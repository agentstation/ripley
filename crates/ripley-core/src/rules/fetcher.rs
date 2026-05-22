use std::path::Path;

use super::registry::{RuleIndex, RuleSourceEntry, SourceRegistry};
use super::{Rule, RuleSource, parse_rule_content};

#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("could not parse index: {0}")]
    ParseIndex(#[from] toml::de::Error),
    #[error("could not write rule file {path}: {source}")]
    WriteFile {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    #[error("could not create directory {path}: {source}")]
    CreateDir {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    #[error("registry error: {0}")]
    Registry(#[from] super::registry::RegistryError),
}

pub async fn fetch_rules(
    source: &RuleSourceEntry,
    data_dir: &Path,
) -> Result<Option<(Vec<Rule>, Option<String>)>, FetchError> {
    let client = reqwest::Client::new();

    let index_url = format!("{}/index.toml", source.url.trim_end_matches('/'));
    let mut request = client.get(&index_url);
    if let Some(ref etag) = source.etag {
        request = request.header("If-None-Match", etag.as_str());
    }

    let response = request.send().await?;
    if response.status() == reqwest::StatusCode::NOT_MODIFIED {
        return Ok(None);
    }

    let new_etag = response
        .headers()
        .get("etag")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let index_response = response.text().await?;
    let index: RuleIndex = toml::from_str(&index_response)?;

    let source_dir = data_dir.join("rules").join("community").join(&source.name);
    std::fs::create_dir_all(&source_dir).map_err(|e| FetchError::CreateDir {
        path: source_dir.clone(),
        source: e,
    })?;

    let mut rules = Vec::new();
    let base_url = source.url.trim_end_matches('/');

    for entry in &index.rules {
        let rule_url = format!("{}/{}", base_url, entry.file);
        let content = match client.get(&rule_url).send().await {
            Ok(resp) => match resp.text().await {
                Ok(text) => text,
                Err(e) => {
                    tracing::warn!("could not fetch rule {}: {e}", entry.file);
                    continue;
                }
            },
            Err(e) => {
                tracing::warn!("could not fetch rule {}: {e}", entry.file);
                continue;
            }
        };

        match parse_rule_content(&content) {
            Ok(mut parsed) => {
                for rule in &mut parsed {
                    rule.source = RuleSource::Community {
                        source_name: source.name.clone(),
                    };
                    if !is_valid_rule(rule) {
                        tracing::warn!("skipping invalid rule: {}", rule.id);
                        continue;
                    }
                }

                let file_path = source_dir.join(&entry.file);
                std::fs::write(&file_path, &content).map_err(|e| FetchError::WriteFile {
                    path: file_path,
                    source: e,
                })?;

                rules.extend(parsed);
            }
            Err(e) => {
                tracing::warn!("could not parse rule {}: {e}", entry.file);
            }
        }
    }

    Ok(Some((rules, new_etag)))
}

pub async fn update_source(
    source: &mut RuleSourceEntry,
    data_dir: &Path,
) -> Result<usize, FetchError> {
    match fetch_rules(source, data_dir).await? {
        Some((rules, new_etag)) => {
            let count = rules.len();
            source.rule_count = count;
            source.last_fetched = Some(chrono::Utc::now().to_rfc3339());
            source.etag = new_etag;
            Ok(count)
        }
        None => {
            tracing::debug!("source '{}' unchanged (304 Not Modified)", source.name);
            Ok(source.rule_count)
        }
    }
}

pub async fn update_all_sources(
    registry: &mut SourceRegistry,
    data_dir: &Path,
) -> Result<Vec<(String, usize)>, FetchError> {
    let mut results = Vec::new();
    for source in &mut registry.sources {
        match update_source(source, data_dir).await {
            Ok(count) => results.push((source.name.clone(), count)),
            Err(e) => {
                tracing::warn!("failed to update source {}: {e}", source.name);
            }
        }
    }
    Ok(results)
}

fn is_valid_rule(rule: &Rule) -> bool {
    if rule.id.is_empty() || rule.name.is_empty() {
        return false;
    }
    for pattern in &rule.patterns {
        if regex::Regex::new(pattern).is_err() {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_rule() {
        let rule = Rule {
            id: "TEST001".to_string(),
            name: "test rule".to_string(),
            description: "test".to_string(),
            ecosystem: "npm".to_string(),
            signal: "test".to_string(),
            weight: crate::types::Severity::High,
            patterns: vec!["curl\\s+".to_string()],
            author: None,
            confidence: None,
            source_attack: None,
            updated_at: None,
            min_ripley_version: None,
            source: RuleSource::Compiled,
        };
        assert!(is_valid_rule(&rule));
    }

    #[test]
    fn test_invalid_rule_empty_id() {
        let rule = Rule {
            id: String::new(),
            name: "test".to_string(),
            description: "test".to_string(),
            ecosystem: "npm".to_string(),
            signal: "test".to_string(),
            weight: crate::types::Severity::High,
            patterns: Vec::new(),
            author: None,
            confidence: None,
            source_attack: None,
            updated_at: None,
            min_ripley_version: None,
            source: RuleSource::Compiled,
        };
        assert!(!is_valid_rule(&rule));
    }

    #[test]
    fn test_invalid_rule_bad_regex() {
        let rule = Rule {
            id: "TEST001".to_string(),
            name: "test".to_string(),
            description: "test".to_string(),
            ecosystem: "npm".to_string(),
            signal: "test".to_string(),
            weight: crate::types::Severity::High,
            patterns: vec!["[invalid".to_string()],
            author: None,
            confidence: None,
            source_attack: None,
            updated_at: None,
            min_ripley_version: None,
            source: RuleSource::Compiled,
        };
        assert!(!is_valid_rule(&rule));
    }
}
