use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

use crate::protocol::{Request, Response, socket_path};

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("no socket path available")]
    NoSocketPath,
    #[error("daemon not running (socket not found)")]
    DaemonNotRunning,
    #[error("connection failed: {0}")]
    Connect(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("empty response from daemon")]
    EmptyResponse,
}

pub async fn send_request(req: &Request) -> Result<Response, ClientError> {
    let path = socket_path().ok_or(ClientError::NoSocketPath)?;

    if !path.exists() {
        return Err(ClientError::DaemonNotRunning);
    }

    let stream = UnixStream::connect(&path).await?;
    let (reader, mut writer) = stream.into_split();

    let mut json = serde_json::to_string(req)?;
    json.push('\n');
    writer.write_all(json.as_bytes()).await?;
    writer.shutdown().await?;

    let mut buf_reader = BufReader::new(reader);
    let mut response_line = String::new();
    buf_reader.read_line(&mut response_line).await?;

    if response_line.is_empty() {
        return Err(ClientError::EmptyResponse);
    }

    let resp: Response = serde_json::from_str(&response_line)?;
    Ok(resp)
}

pub fn is_daemon_running() -> bool {
    socket_path().is_some_and(|p| p.exists())
}
