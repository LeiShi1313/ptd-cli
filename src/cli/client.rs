use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::time::timeout;

use crate::shared::protocol::{HostMessage, RequestMessage, ResponseMessage};

/// Send a request to the daemon via an instance's Unix socket and wait for the response.
pub async fn send_request(
    socket_path: &Path,
    method: &str,
    params: serde_json::Value,
    timeout_secs: u64,
) -> Result<ResponseMessage> {
    let stream = UnixStream::connect(socket_path)
        .await
        .context("failed to connect to daemon socket")?;

    let (reader, mut writer) = stream.into_split();

    let request = HostMessage::Request(RequestMessage {
        id: uuid::Uuid::new_v4().to_string(),
        method: method.to_string(),
        params,
    });

    let json = serde_json::to_string(&request).context("failed to serialize request")?;
    writer.write_all(json.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;

    // Read one response line
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    let read_result = timeout(Duration::from_secs(timeout_secs), reader.read_line(&mut line))
        .await
        .context("request timed out")?
        .context("failed to read response")?;

    if read_result == 0 {
        anyhow::bail!("daemon closed connection without responding");
    }

    let msg: HostMessage =
        serde_json::from_str(line.trim()).context("failed to parse response")?;

    match msg {
        HostMessage::Response(resp) => Ok(resp),
        _ => anyhow::bail!("expected response, got unexpected message type"),
    }
}
