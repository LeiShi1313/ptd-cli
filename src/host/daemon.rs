use anyhow::{Context, Result};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info, warn};

use std::sync::Arc;

use crate::shared::constants::{ALLOWED_METHODS, HELLO_TIMEOUT_SECS};
use crate::shared::paths;
use crate::shared::protocol::{HostMessage, RequestMessage, ResponseMessage};

use super::native_messaging;
use super::registry;
use super::router::Router;

/// Start the daemon: read hello from browser, publish socket, bridge traffic.
pub async fn run() -> Result<()> {
    let mut stdin = io::stdin();
    let stdout = Arc::new(Mutex::new(io::stdout()));

    // Step 1: Wait for hello from the extension
    info!("waiting for hello from extension...");
    let hello = timeout(Duration::from_secs(HELLO_TIMEOUT_SECS), async {
        native_messaging::read_message(&mut stdin).await
    })
    .await
    .context("hello timeout")??
    .context("stdin closed before hello")?;

    let hello = match hello {
        HostMessage::Hello(h) => h,
        other => anyhow::bail!("expected hello, got: {:?}", other),
    };

    let instance_id = hello.instance_id.clone();
    info!(instance_id = %instance_id, browser = %hello.browser, "received hello");

    // Step 2: Publish registry and socket
    registry::publish(&hello)?;
    let socket_path = paths::instance_socket_path(&instance_id);
    let listener = UnixListener::bind(&socket_path)
        .context("failed to bind instance socket")?;
    info!(path = %socket_path.display(), "socket published");

    // Step 3: Run the main select loop
    let router = Arc::new(Mutex::new(Router::new()));
    let result = main_loop(&mut stdin, stdout.clone(), listener, router.clone()).await;

    // Step 4: Cleanup
    info!("shutting down, cleaning up...");
    router.lock().await.fail_all("daemon shutting down");
    registry::cleanup(&instance_id);

    result
}

async fn main_loop(
    stdin: &mut io::Stdin,
    stdout: Arc<Mutex<io::Stdout>>,
    listener: UnixListener,
    router: Arc<Mutex<Router>>,
) -> Result<()> {
    loop {
        tokio::select! {
            // Branch 1: Message from browser via stdin (Native Messaging)
            msg = native_messaging::read_message(stdin) => {
                match msg {
                    Ok(Some(HostMessage::Response(resp))) => {
                        debug!(id = %resp.id, "response from extension");
                        router.lock().await.deliver(resp);
                    }
                    Ok(Some(other)) => {
                        warn!("unexpected message from browser: {:?}", other);
                    }
                    Ok(None) => {
                        info!("browser closed stdin, exiting");
                        return Ok(());
                    }
                    Err(e) => {
                        error!("error reading from browser: {e:#}");
                        return Err(e);
                    }
                }
            }

            // Branch 2: New CLI client connection on socket
            accept = listener.accept() => {
                let (stream, _addr) = accept.context("failed to accept CLI connection")?;
                debug!("new CLI client connected");
                let router = router.clone();
                let stdout = stdout.clone();

                // Handle each CLI client in a spawned task
                tokio::spawn(async move {
                    if let Err(e) = handle_cli_client(stream, router, stdout).await {
                        warn!("CLI client error: {e:#}");
                    }
                });
            }
        }
    }
}

async fn handle_cli_client(
    stream: tokio::net::UnixStream,
    router: Arc<Mutex<Router>>,
    stdout: Arc<Mutex<io::Stdout>>,
) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    // Read one NDJSON line from the CLI
    reader
        .read_line(&mut line)
        .await
        .context("failed to read from CLI")?;

    if line.is_empty() {
        return Ok(()); // Client disconnected
    }

    let msg: HostMessage =
        serde_json::from_str(line.trim()).context("failed to parse CLI message")?;

    let request = match msg {
        HostMessage::Request(r) => r,
        _ => {
            let err_resp = HostMessage::Response(ResponseMessage::error(
                "unknown".into(),
                "PARSE_ERROR",
                "expected a request message",
            ));
            let json = serde_json::to_string(&err_resp)?;
            writer.write_all(json.as_bytes()).await?;
            writer.write_all(b"\n").await?;
            return Ok(());
        }
    };

    // Validate method against allowlist
    if !ALLOWED_METHODS.contains(&request.method.as_str()) {
        let resp = HostMessage::Response(ResponseMessage::error(
            request.id,
            "METHOD_NOT_ALLOWED",
            format!("method '{}' is not in the allowlist", request.method),
        ));
        let json = serde_json::to_string(&resp)?;
        writer.write_all(json.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        return Ok(());
    }

    // Register in router and forward to browser
    let rx = router.lock().await.register(request.id.clone());

    let forward_msg = HostMessage::Request(RequestMessage {
        id: request.id.clone(),
        method: request.method,
        params: request.params,
    });
    {
        let mut stdout = stdout.lock().await;
        native_messaging::write_message(&mut *stdout, &forward_msg).await?;
    }

    // Wait for response from browser (routed back via router)
    let response = rx.await.context("router channel closed")?;

    // Send response back to CLI client
    let resp_msg = HostMessage::Response(response);
    let json = serde_json::to_string(&resp_msg)?;
    writer.write_all(json.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;

    Ok(())
}
