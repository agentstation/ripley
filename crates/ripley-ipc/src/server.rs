use std::future::Future;
use std::path::Path;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio_util::sync::CancellationToken;

use crate::protocol::{Request, Response};

#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialize(#[from] serde_json::Error),
}

pub async fn serve<F, Fut>(
    socket_path: &Path,
    handler: F,
    cancel: CancellationToken,
) -> Result<(), ServerError>
where
    F: Fn(Request) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Response> + Send,
{
    if socket_path.exists() {
        std::fs::remove_file(socket_path)?;
    }

    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let listener = UnixListener::bind(socket_path)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(socket_path, perms)?;
    }

    tracing::info!("IPC server listening on {}", socket_path.display());

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                tracing::info!("IPC server shutting down");
                let _ = std::fs::remove_file(socket_path);
                return Ok(());
            }
            accept = listener.accept() => {
                let (stream, _) = accept?;
                let (reader, mut writer) = stream.into_split();

                let mut buf_reader = BufReader::new(reader);
                let mut line = String::new();

                if buf_reader.read_line(&mut line).await? == 0 {
                    continue;
                }

                let request: Request = match serde_json::from_str(&line) {
                    Ok(r) => r,
                    Err(e) => {
                        tracing::warn!("invalid IPC request: {e}");
                        let resp = Response::Error(format!("invalid request: {e}"));
                        let mut json = serde_json::to_string(&resp)?;
                        json.push('\n');
                        let _ = writer.write_all(json.as_bytes()).await;
                        continue;
                    }
                };

                let response = handler(request).await;
                let mut json = serde_json::to_string(&response)?;
                json.push('\n');
                let _ = writer.write_all(json.as_bytes()).await;
            }
        }
    }
}
