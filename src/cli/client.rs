use std::time::Duration;

use anyhow::{Context, Result};
use interprocess::local_socket::tokio::{prelude::*, Stream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::time::timeout;

use crate::shared::protocol::{HostMessage, RequestMessage, ResponseMessage};

/// Send a request to the daemon via an instance's IPC endpoint and wait for the response.
///
/// On Unix, `ipc_name` is a filesystem socket path.
/// On Windows, `ipc_name` is a named pipe identifier.
pub async fn send_request(
    ipc_name: &str,
    method: &str,
    params: serde_json::Value,
    timeout_secs: u64,
) -> Result<ResponseMessage> {
    let name = create_ipc_name(ipc_name)?;
    let stream = Stream::connect(name)
        .await
        .context("failed to connect to daemon")?;

    let (reader, writer) = tokio::io::split(stream);
    let mut writer = writer;

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

/// Create a platform-appropriate IPC name.
///
/// On Unix, uses filesystem path via `GenericFilePath`.
/// On Windows, uses namespaced pipe name via `GenericNamespaced`.
fn create_ipc_name(ipc_name: &str) -> Result<interprocess::local_socket::Name<'_>> {
    #[cfg(unix)]
    {
        use interprocess::local_socket::{GenericFilePath, ToFsName};
        ipc_name
            .to_fs_name::<GenericFilePath>()
            .context("failed to create IPC name")
    }
    #[cfg(windows)]
    {
        use interprocess::local_socket::{GenericNamespaced, ToNsName};
        ipc_name
            .to_ns_name::<GenericNamespaced>()
            .context("failed to create IPC name")
    }
}
