pub mod client;
pub mod ipc_client;
pub mod types;

pub use client::ClashClient;

use async_trait::async_trait;
use std::collections::HashMap;

use crate::clash::types::*;

/// Common interface for Clash API communication.
/// Phase 1: HttpClashClient (direct HTTP)
/// Phase 3: IpcClashClient (via daemon IPC)
#[async_trait]
pub trait ClashApi: Send + Sync {
    async fn get_proxies(&self) -> anyhow::Result<HashMap<String, ProxyGroup>>;
    async fn switch_proxy(&self, group: &str, proxy: &str) -> anyhow::Result<()>;
    async fn get_traffic(&self) -> anyhow::Result<Traffic>;
    async fn get_memory(&self) -> anyhow::Result<u64>;
    async fn get_version(&self) -> anyhow::Result<VersionInfo>;
    async fn get_configs(&self) -> anyhow::Result<ClashConfigs>;
    async fn test_latency(
        &self,
        proxy: &str,
        url: &str,
        timeout: u16,
    ) -> anyhow::Result<LatencyResult>;
    async fn get_connections(&self) -> anyhow::Result<Vec<Connection>>;
    async fn close_connection(&self, id: &str) -> anyhow::Result<()>;
    async fn close_all_connections(&self) -> anyhow::Result<()>;
    async fn get_logs(&self) -> anyhow::Result<Vec<LogEntry>>;
    async fn get_rules(&self) -> anyhow::Result<Vec<Rule>>;
    async fn set_config_mode(&self, mode: &str) -> anyhow::Result<()>;
    async fn set_tun(&self, enable: bool) -> anyhow::Result<()>;
    async fn reload_config(&self, config_path: &str) -> anyhow::Result<()>;
    async fn refresh_all(&self) -> anyhow::Result<RefreshData>;
}

/// Combined refresh data used by the background task
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RefreshData {
    pub proxy_groups: HashMap<String, ProxyGroup>,
    pub memory: u64,
    pub mode: String,
    pub upload_speed: u64,
    pub download_speed: u64,
    pub active_conn_count: usize,
    pub core_version: String,
    pub connections: Vec<Connection>,
    pub logs: Vec<LogEntry>,
    pub rules: Vec<Rule>,
    #[serde(default)]
    pub api_reachable: bool,
    #[serde(default)]
    pub tun_enabled: bool,
}
