use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use ripley_ipc::protocol::{GuardDecision, GuardPromptData};
use serde::{Deserialize, Serialize};
use specta::Type;
use tokio::sync::{Mutex, oneshot};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GuardEventPayload {
    pub id: String,
    pub package: String,
    pub version: String,
    pub script: String,
    pub risk_level: String,
    pub matched_rules: Vec<String>,
}

impl GuardEventPayload {
    pub fn from_prompt(id: String, prompt: GuardPromptData) -> Self {
        Self {
            id,
            package: prompt.package,
            version: prompt.version,
            script: prompt.script,
            risk_level: prompt.risk_level,
            matched_rules: prompt.matched_rules,
        }
    }
}

pub type PendingMap = Arc<Mutex<HashMap<String, oneshot::Sender<GuardDecision>>>>;

pub fn new_pending() -> PendingMap {
    Arc::new(Mutex::new(HashMap::new()))
}

static ID_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn next_id() -> String {
    let n = ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("guard-{n}")
}

#[cfg(unix)]
pub use unix::{BridgeError, serve};

#[cfg(unix)]
mod unix {
    use std::path::Path;

    use ripley_ipc::protocol::{Request, Response};
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::{UnixListener, UnixStream};
    use tokio::sync::oneshot;
    use tokio_util::sync::CancellationToken;

    use super::{GuardEventPayload, PendingMap, next_id};

    #[derive(Debug, thiserror::Error)]
    pub enum BridgeError {
        #[error("io: {0}")]
        Io(#[from] std::io::Error),
        #[error("serialize: {0}")]
        Serialize(#[from] serde_json::Error),
    }

    pub async fn serve<F>(
        socket_path: &Path,
        pending: PendingMap,
        emit: F,
        cancel: CancellationToken,
    ) -> Result<(), BridgeError>
    where
        F: Fn(GuardEventPayload) + Send + Sync + Clone + 'static,
    {
        if socket_path.exists() {
            std::fs::remove_file(socket_path)?;
        }
        if let Some(parent) = socket_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let listener = UnixListener::bind(socket_path)?;

        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(socket_path, perms)?;
        }

        tracing::info!("guard bridge listening on {}", socket_path.display());

        loop {
            tokio::select! {
                _ = cancel.cancelled() => {
                    let _ = std::fs::remove_file(socket_path);
                    return Ok(());
                }
                accept = listener.accept() => {
                    let (stream, _) = accept?;
                    let pending = pending.clone();
                    let emit = emit.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_conn(stream, pending, emit).await {
                            tracing::warn!("guard bridge connection error: {e}");
                        }
                    });
                }
            }
        }
    }

    async fn handle_conn<F>(
        stream: UnixStream,
        pending: PendingMap,
        emit: F,
    ) -> Result<(), BridgeError>
    where
        F: Fn(GuardEventPayload) + Send + Sync + 'static,
    {
        let (reader, mut writer) = stream.into_split();
        let mut buf_reader = BufReader::new(reader);
        let mut line = String::new();

        if buf_reader.read_line(&mut line).await? == 0 {
            return Ok(());
        }

        let request: Request = match serde_json::from_str(line.trim()) {
            Ok(r) => r,
            Err(e) => {
                let resp = Response::Error(format!("invalid request: {e}"));
                write_response(&mut writer, &resp).await?;
                return Ok(());
            }
        };

        let response = match request {
            Request::GuardPrompt(prompt) => {
                let id = next_id();
                let (tx, rx) = oneshot::channel();
                pending.lock().await.insert(id.clone(), tx);
                tracing::info!(id = %id, "guard event_received");
                emit(GuardEventPayload::from_prompt(id.clone(), prompt));
                tracing::info!(id = %id, "guard event_emitted");
                match rx.await {
                    Ok(decision) => Response::GuardDecision(decision),
                    Err(_) => {
                        pending.lock().await.remove(&id);
                        Response::Error("guard dialog cancelled".into())
                    }
                }
            }
            _ => Response::Error("desktop bridge handles GuardPrompt only".into()),
        };

        write_response(&mut writer, &response).await?;
        Ok(())
    }

    async fn write_response(
        writer: &mut tokio::net::unix::OwnedWriteHalf,
        response: &Response,
    ) -> Result<(), BridgeError> {
        let mut json = serde_json::to_string(response)?;
        json.push('\n');
        writer.write_all(json.as_bytes()).await?;
        Ok(())
    }
}
