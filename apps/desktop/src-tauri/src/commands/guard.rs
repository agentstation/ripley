use ripley_ipc::protocol::GuardDecision;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use crate::ipc_bridge::PendingMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub enum Decision {
    Allow,
    Block,
    Trust,
}

impl From<Decision> for GuardDecision {
    fn from(d: Decision) -> Self {
        match d {
            Decision::Allow => Self::Allow,
            Decision::Block => Self::Block,
            Decision::Trust => Self::Trust,
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn submit_guard_decision(
    pending: State<'_, PendingMap>,
    id: String,
    decision: Decision,
) -> Result<(), String> {
    let mut map = pending.lock().await;
    let tx = map
        .remove(&id)
        .ok_or_else(|| format!("unknown guard id: {id}"))?;
    drop(map);
    tx.send(decision.into())
        .map_err(|_| "guard receiver dropped".to_string())?;
    Ok(())
}
