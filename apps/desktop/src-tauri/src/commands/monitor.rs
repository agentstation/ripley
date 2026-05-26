use std::io::{BufRead, BufReader};

use ripley_core::dirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct MonitorEntryDto {
    pub timestamp: String,
    pub event_type: String,
    pub severity: String,
    pub process: String,
    pub pid: Option<u32>,
    pub reason: String,
    pub detail: String,
    pub action: String,
}

#[tauri::command]
#[specta::specta]
pub fn list_monitor_events(limit: u32) -> Result<Vec<MonitorEntryDto>, String> {
    let data_dir = dirs::data_dir().map_err(|e| e.to_string())?;
    let log_path = data_dir.join("guard.jsonl");

    if !log_path.exists() {
        return Ok(Vec::new());
    }

    let file = std::fs::File::open(&log_path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader
        .lines()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let entries: Vec<MonitorEntryDto> = lines
        .into_iter()
        .rev()
        .filter_map(|line| parse_monitor(&line))
        .take(limit as usize)
        .collect();

    Ok(entries)
}

fn parse_monitor(line: &str) -> Option<MonitorEntryDto> {
    let raw: serde_json::Value = serde_json::from_str(line).ok()?;
    let source = raw.get("source").and_then(|v| v.as_str()).unwrap_or("");
    let event_type = raw.get("event_type").and_then(|v| v.as_str()).unwrap_or("");

    let is_monitor = source.contains("monitor") || !event_type.is_empty();
    if !is_monitor {
        return None;
    }

    Some(MonitorEntryDto {
        timestamp: raw
            .get("timestamp")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        event_type: event_type.to_string(),
        severity: raw
            .get("severity")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        process: raw
            .get("process")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        pid: raw.get("pid").and_then(|v| v.as_u64()).map(|p| p as u32),
        reason: raw
            .get("reason")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        detail: raw
            .get("detail")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        action: raw
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_monitor_event() {
        let line = r#"{"timestamp":"2026-05-26T00:00:00Z","event_type":"process_alert","severity":"high","process":"node","pid":1234,"reason":"new_connection","detail":"evil.com","action":"observed"}"#;
        let entry = parse_monitor(line).expect("parse");
        assert_eq!(entry.event_type, "process_alert");
        assert_eq!(entry.pid, Some(1234));
        assert_eq!(entry.process, "node");
    }

    #[test]
    fn skips_non_monitor_lines() {
        let line =
            r#"{"timestamp":"2026-05-26T00:00:00Z","source":"script-shell","action":"blocked"}"#;
        assert!(parse_monitor(line).is_none());
    }
}
