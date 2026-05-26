use std::io::{BufRead, BufReader};

use ripley_core::dirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, specta::Type)]
pub struct GuardLogEntry {
    pub timestamp: String,
    pub package: String,
    pub version: String,
    pub script: String,
    pub action: String,
    pub risk_level: String,
    pub matched_rules: Vec<String>,
    pub source: String,
    pub user_decision: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct GuardLogPage {
    pub entries: Vec<GuardLogEntry>,
    pub total: u32,
}

#[tauri::command]
#[specta::specta]
pub fn list_guard_log(offset: u32, limit: u32) -> Result<GuardLogPage, String> {
    let data_dir = dirs::data_dir().map_err(|e| e.to_string())?;
    let log_path = data_dir.join("guard.jsonl");

    if !log_path.exists() {
        return Ok(GuardLogPage {
            entries: Vec::new(),
            total: 0,
        });
    }

    let file = std::fs::File::open(&log_path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader
        .lines()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let total = lines.len() as u32;
    let offset = offset as usize;
    let limit = limit as usize;

    let entries: Vec<GuardLogEntry> = lines
        .into_iter()
        .rev()
        .skip(offset)
        .take(limit)
        .filter_map(|line| parse_entry(&line))
        .collect();

    Ok(GuardLogPage { entries, total })
}

fn parse_entry(line: &str) -> Option<GuardLogEntry> {
    let raw: serde_json::Value = serde_json::from_str(line).ok()?;
    let matched_rules = raw
        .get("matched_rules")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    Some(GuardLogEntry {
        timestamp: raw
            .get("timestamp")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        package: raw
            .get("package")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        version: raw
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        script: raw
            .get("script")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        action: raw
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        risk_level: raw
            .get("risk_level")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        matched_rules,
        source: raw
            .get("source")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        user_decision: raw.get("user_decision").and_then(|v| v.as_bool()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_canonical_entry() {
        let line = r#"{"timestamp":"2026-05-26T00:00:00Z","package":"left-pad","version":"1.3.0","script":"postinstall","action":"blocked","risk_level":"high","matched_rules":["NET001"],"source":"script-shell","user_decision":true}"#;
        let entry = parse_entry(line).expect("parse");
        assert_eq!(entry.package, "left-pad");
        assert_eq!(entry.action, "blocked");
        assert_eq!(entry.matched_rules, vec!["NET001".to_string()]);
        assert_eq!(entry.user_decision, Some(true));
    }

    #[test]
    fn skips_malformed_lines() {
        assert!(parse_entry("not json").is_none());
    }

    #[test]
    fn missing_fields_default_to_empty_strings() {
        let entry = parse_entry("{}").expect("parse");
        assert_eq!(entry.package, "");
        assert!(entry.matched_rules.is_empty());
        assert!(entry.user_decision.is_none());
    }
}
