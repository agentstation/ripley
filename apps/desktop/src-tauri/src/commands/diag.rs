use std::io::Write as _;
use std::path::PathBuf;

use serde::Serialize;
use tauri::State;

use crate::ipc_bridge::LatencyMap;

#[derive(Serialize)]
struct LatencyRecord {
    id: String,
    elapsed_ms: u128,
    recorded_at_ms: u128,
}

#[tauri::command]
#[specta::specta]
pub async fn report_visible(latencies: State<'_, LatencyMap>, id: String) -> Result<(), String> {
    let start = {
        let mut map = latencies.lock().await;
        map.remove(&id)
    };
    let Some(start) = start else {
        tracing::warn!(id = %id, "report_visible called but no start instant recorded");
        return Ok(());
    };

    let elapsed = start.elapsed();
    let elapsed_ms = elapsed.as_millis();
    tracing::info!(id = %id, elapsed_ms = elapsed_ms as u64, "guard dialog_visible");

    let path = match latency_log_path() {
        Some(p) => p,
        None => return Ok(()),
    };
    let recorded_at_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let record = LatencyRecord {
        id,
        elapsed_ms,
        recorded_at_ms,
    };
    if let Err(e) = append_record(&path, &record) {
        tracing::warn!("failed to append guard-latency record: {e}");
    }
    Ok(())
}

fn latency_log_path() -> Option<PathBuf> {
    let data_dir = ripley_core::dirs::data_dir().ok()?;
    Some(data_dir.join("guard-latency.jsonl"))
}

fn append_record(path: &PathBuf, record: &LatencyRecord) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut json = serde_json::to_string(record)?;
    json.push('\n');
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}
