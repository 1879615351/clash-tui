use anyhow::Context;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

use super::protocol::{IpcRequest, IpcResponse};

/// IPC client that connects to the daemon over TCP.
pub struct IpcClient {
    stream: BufReader<TcpStream>,
}

impl IpcClient {
    pub async fn connect(port: u16) -> anyhow::Result<Self> {
        let addr = format!("127.0.0.1:{}", port);
        let stream = TcpStream::connect(&addr)
            .await
            .context("Failed to connect to daemon")?;
        tracing::info!("Connected to daemon on {}", addr);
        Ok(Self {
            stream: BufReader::new(stream),
        })
    }

    pub async fn send_request(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        let request = IpcRequest {
            id: rand_id(),
            method: method.to_string(),
            params,
        };

        let mut json = serde_json::to_string(&request)?;
        json.push('\n');

        self.stream.get_mut().write_all(json.as_bytes()).await?;

        let mut line = String::new();
        self.stream.read_line(&mut line).await?;

        let response: IpcResponse = serde_json::from_str(&line)?;

        if let Some(error) = response.error {
            anyhow::bail!("Daemon error: {}", error);
        }

        Ok(response.ok.unwrap_or(serde_json::Value::Null))
    }
}

fn rand_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}
