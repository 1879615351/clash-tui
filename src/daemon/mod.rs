use std::net::TcpListener;
use std::sync::Arc;

use tokio::process::Command as AsyncCommand;

use crate::clash::client::ClashClient;
use crate::clash::ClashApi;
use crate::config::AppConfig;
use crate::ipc::protocol::IpcRequest;

static DAEMON_PORT_FILE: &str = "daemon.port";

pub struct Daemon {
    clash_client: Arc<ClashClient>,
    port: u16,
    core_path: Option<String>,
    core_process: Option<tokio::process::Child>,
}

impl Daemon {
    pub async fn new(config: AppConfig) -> anyhow::Result<Self> {
        let clash_client = Arc::new(ClashClient::new(
            &config.api.host,
            config.api.port,
            config.api.secret.clone(),
        ));

        let port = find_free_port()?;

        let port_path = AppConfig::config_dir()?.join(DAEMON_PORT_FILE);
        std::fs::write(&port_path, port.to_string())?;
        tracing::info!("Daemon port {} written to {}", port, port_path.display());

        // Try to spawn the core process
        let core_path = if config.core.core_path.is_empty() {
            None
        } else {
            Some(config.core.core_path.clone())
        };

        Ok(Self {
            clash_client,
            port,
            core_path,
            core_process: None,
        })
    }

    /// Start the mihomo core process.
    async fn start_core(&mut self) -> anyhow::Result<()> {
        let core_path = match &self.core_path {
            Some(p) if !p.is_empty() => p.clone(),
            _ => {
                // Try to install embedded core
                match crate::core::CoreManager::install() {
                    Ok(p) => p.display().to_string(),
                    Err(_) => {
                        tracing::info!("No core path configured, skipping core start");
                        return Ok(());
                    }
                }
            }
        };

        let config_dir = AppConfig::config_dir()?;
        let work_dir = config_dir.join("clash");

        // Ensure config directory exists
        std::fs::create_dir_all(&work_dir)?;

        tracing::info!(
            "Starting core: {} (workdir: {})",
            core_path,
            work_dir.display()
        );

        let child = AsyncCommand::new(&core_path)
            .arg("-d")
            .arg(&work_dir)
            .kill_on_drop(true)
            .spawn();

        match child {
            Ok(process) => {
                tracing::info!("Core started (PID: {:?})", process.id());
                self.core_process = Some(process);
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to start core: {}", e);
                Err(e.into())
            }
        }
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        self.start_core().await?;

        tracing::info!("Starting daemon on port {}...", self.port);

        let client = self.clash_client.clone();

        let handler: crate::ipc::server::RequestHandler = Arc::new(move |request| {
            let c = client.clone();
            Box::pin(async move { dispatch_request(&c, request).await })
        });

        crate::ipc::server::run_server(self.port, handler).await
    }
}

impl Drop for Daemon {
    fn drop(&mut self) {
        if let Some(ref mut child) = self.core_process {
            tracing::info!("Stopping core process...");
            let _ = child.start_kill();
        }
    }
}

async fn dispatch_request(
    client: &ClashClient,
    request: IpcRequest,
) -> anyhow::Result<serde_json::Value> {
    match request.method.as_str() {
        "get_proxies" => {
            let proxies = client.get_proxies().await?;
            Ok(serde_json::to_value(&proxies)?)
        }
        "switch_proxy" => {
            let group = request.params["group"].as_str().unwrap_or("");
            let proxy = request.params["proxy"].as_str().unwrap_or("");
            client.switch_proxy(group, proxy).await?;
            Ok(serde_json::Value::Null)
        }
        "get_traffic" => {
            let traffic = client.get_traffic().await?;
            Ok(serde_json::to_value(&traffic)?)
        }
        "get_memory" => {
            let mem = client.get_memory().await?;
            Ok(serde_json::to_value(mem)?)
        }
        "get_version" => {
            let v = client.get_version().await?;
            Ok(serde_json::to_value(&v)?)
        }
        "get_configs" => {
            let c = client.get_configs().await?;
            Ok(serde_json::to_value(&c)?)
        }
        "test_latency" => {
            let proxy = request.params["proxy"].as_str().unwrap_or("");
            let url = request.params["url"]
                .as_str()
                .unwrap_or("https://www.gstatic.com/generate_204");
            let timeout = request.params["timeout"].as_u64().unwrap_or(5000) as u16;
            let r = client.test_latency(proxy, url, timeout).await?;
            Ok(serde_json::to_value(&r)?)
        }
        "get_connections" => {
            let conns = client.get_connections().await?;
            Ok(serde_json::to_value(&conns)?)
        }
        "close_connection" => {
            let id = request.params["id"].as_str().unwrap_or("");
            client.close_connection(id).await?;
            Ok(serde_json::Value::Null)
        }
        "close_all_connections" => {
            client.close_all_connections().await?;
            Ok(serde_json::Value::Null)
        }
        "get_logs" => {
            let logs = client.get_logs().await?;
            Ok(serde_json::to_value(&logs)?)
        }
        "get_rules" => {
            let rules = client.get_rules().await?;
            Ok(serde_json::to_value(&rules)?)
        }
        "set_config_mode" => {
            let mode = request.params["mode"].as_str().unwrap_or("rule");
            client.set_config_mode(mode).await?;
            Ok(serde_json::Value::Null)
        }
        "reload_config" => {
            let path = request.params["path"].as_str().unwrap_or("");
            client.reload_config(path).await?;
            Ok(serde_json::Value::Null)
        }
        "refresh_all" => {
            let data = client.refresh_all().await?;
            Ok(serde_json::to_value(&data)?)
        }
        _ => anyhow::bail!("Unknown method: {}", request.method),
    }
}

fn find_free_port() -> anyhow::Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    Ok(listener.local_addr()?.port())
}
