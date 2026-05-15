use std::collections::HashMap;

use async_trait::async_trait;

use super::types::*;
use super::{ClashApi, RefreshData};
use crate::ipc::client::IpcClient;

/// IPC-based Clash API client (Phase 3: Client/Daemon mode)
pub struct IpcClashClient {
    ipc: tokio::sync::Mutex<IpcClient>,
}

impl IpcClashClient {
    pub async fn connect(port: u16) -> anyhow::Result<Self> {
        let ipc = IpcClient::connect(port).await?;
        Ok(Self {
            ipc: tokio::sync::Mutex::new(ipc),
        })
    }
}

#[async_trait]
impl ClashApi for IpcClashClient {
    async fn get_proxies(&self) -> anyhow::Result<HashMap<String, ProxyGroup>> {
        let resp = self
            .ipc
            .lock()
            .await
            .send_request("get_proxies", serde_json::json!({}))
            .await?;
        Ok(serde_json::from_value(resp)?)
    }

    async fn switch_proxy(&self, group: &str, proxy: &str) -> anyhow::Result<()> {
        self.ipc
            .lock()
            .await
            .send_request(
                "switch_proxy",
                serde_json::json!({
                    "group": group,
                    "proxy": proxy,
                }),
            )
            .await?;
        Ok(())
    }

    async fn get_traffic(&self) -> anyhow::Result<Traffic> {
        let resp = self
            .ipc
            .lock()
            .await
            .send_request("get_traffic", serde_json::json!({}))
            .await?;
        Ok(serde_json::from_value(resp)?)
    }

    async fn get_memory(&self) -> anyhow::Result<u64> {
        let resp = self
            .ipc
            .lock()
            .await
            .send_request("get_memory", serde_json::json!({}))
            .await?;
        Ok(serde_json::from_value(resp)?)
    }

    async fn get_version(&self) -> anyhow::Result<VersionInfo> {
        let resp = self
            .ipc
            .lock()
            .await
            .send_request("get_version", serde_json::json!({}))
            .await?;
        Ok(serde_json::from_value(resp)?)
    }

    async fn get_configs(&self) -> anyhow::Result<ClashConfigs> {
        let resp = self
            .ipc
            .lock()
            .await
            .send_request("get_configs", serde_json::json!({}))
            .await?;
        Ok(serde_json::from_value(resp)?)
    }

    async fn test_latency(
        &self,
        proxy: &str,
        url: &str,
        timeout: u16,
    ) -> anyhow::Result<LatencyResult> {
        let resp = self
            .ipc
            .lock()
            .await
            .send_request(
                "test_latency",
                serde_json::json!({
                    "proxy": proxy,
                    "url": url,
                    "timeout": timeout,
                }),
            )
            .await?;
        Ok(serde_json::from_value(resp)?)
    }

    async fn get_connections(&self) -> anyhow::Result<Vec<Connection>> {
        let resp = self
            .ipc
            .lock()
            .await
            .send_request("get_connections", serde_json::json!({}))
            .await?;
        Ok(serde_json::from_value(resp)?)
    }

    async fn close_connection(&self, id: &str) -> anyhow::Result<()> {
        self.ipc
            .lock()
            .await
            .send_request("close_connection", serde_json::json!({"id": id}))
            .await?;
        Ok(())
    }

    async fn close_all_connections(&self) -> anyhow::Result<()> {
        self.ipc
            .lock()
            .await
            .send_request("close_all_connections", serde_json::json!({}))
            .await?;
        Ok(())
    }

    async fn get_logs(&self) -> anyhow::Result<Vec<LogEntry>> {
        let resp = self
            .ipc
            .lock()
            .await
            .send_request("get_logs", serde_json::json!({}))
            .await?;
        Ok(serde_json::from_value(resp)?)
    }

    async fn get_rules(&self) -> anyhow::Result<Vec<Rule>> {
        let resp = self
            .ipc
            .lock()
            .await
            .send_request("get_rules", serde_json::json!({}))
            .await?;
        Ok(serde_json::from_value(resp)?)
    }

    async fn set_config_mode(&self, mode: &str) -> anyhow::Result<()> {
        self.ipc
            .lock()
            .await
            .send_request("set_config_mode", serde_json::json!({"mode": mode}))
            .await?;
        Ok(())
    }

    async fn reload_config(&self, config_path: &str) -> anyhow::Result<()> {
        self.ipc
            .lock()
            .await
            .send_request("reload_config", serde_json::json!({"path": config_path}))
            .await?;
        Ok(())
    }

    async fn refresh_all(&self) -> anyhow::Result<RefreshData> {
        let resp = self
            .ipc
            .lock()
            .await
            .send_request("refresh_all", serde_json::json!({}))
            .await?;
        let mut data: RefreshData = serde_json::from_value(resp)?;
        data.api_reachable = !data.core_version.is_empty();
        Ok(data)
    }
}
