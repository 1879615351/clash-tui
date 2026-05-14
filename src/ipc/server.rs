use anyhow::Context;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

use super::protocol::{IpcRequest, IpcResponse};
use std::sync::Arc;

/// Async handler: takes a request and returns JSON value or error.
pub type RequestHandler = Arc<
    dyn Fn(
            IpcRequest,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = anyhow::Result<serde_json::Value>> + Send>,
        > + Send
        + Sync,
>;

/// Run a TCP-based IPC server.
pub async fn run_server(port: u16, handler: RequestHandler) -> anyhow::Result<()> {
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr)
        .await
        .context("Failed to bind daemon port")?;
    tracing::info!("Daemon listening on {}", addr);

    loop {
        match listener.accept().await {
            Ok((stream, peer)) => {
                tracing::info!("Client connected: {}", peer);
                let h = handler.clone();
                tokio::spawn(async move {
                    handle_client(stream, h).await;
                });
            }
            Err(e) => {
                tracing::error!("Accept failed: {}", e);
            }
        }
    }
}

async fn handle_client(stream: TcpStream, handler: RequestHandler) {
    let (reader, mut writer) = stream.into_split();
    let mut buf_reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        match buf_reader.read_line(&mut line).await {
            Ok(0) => break,
            Ok(_) => match serde_json::from_str::<IpcRequest>(&line) {
                Ok(request) => {
                    let id = request.id;
                    let result = handler(request).await;
                    let response = match result {
                        Ok(value) => IpcResponse::success(id, value),
                        Err(e) => IpcResponse::failure(id, format!("{}", e)),
                    };
                    let mut json = serde_json::to_string(&response).unwrap_or_default();
                    json.push('\n');
                    if writer.write_all(json.as_bytes()).await.is_err() {
                        break;
                    }
                }
                Err(e) => {
                    tracing::warn!("Invalid request: {}", e);
                }
            },
            Err(e) => {
                tracing::error!("Read error: {}", e);
                break;
            }
        }
    }
    tracing::info!("Client disconnected");
}
