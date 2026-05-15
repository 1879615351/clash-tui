use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Response from GET /proxies
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProxiesResponse {
    pub proxies: HashMap<String, ProxyGroup>,
}

/// A proxy group or individual proxy node.
/// In mihomo's /proxies response, both groups (Selector, URLTest) and
/// individual nodes (ss, vmess) share this same structure.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyGroup {
    pub name: String,
    #[serde(rename = "type")]
    pub group_type: String,
    #[serde(default)]
    pub now: Option<String>,
    #[serde(default)]
    pub all: Vec<String>,
    #[serde(default)]
    pub udp: bool,
    #[serde(default)]
    pub history: Option<Vec<LatencyRecord>>,
    #[serde(default)]
    pub hidden: bool,
}

/// A single proxy node (returned in the detailed proxies map)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Proxy {
    pub name: String,
    #[serde(rename = "type")]
    pub proxy_type: String,
    #[serde(default)]
    pub udp: bool,
    #[serde(default)]
    pub history: Option<Vec<LatencyRecord>>,
    #[serde(default)]
    pub hidden: bool,
}

/// Latency history record
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LatencyRecord {
    pub time: String,
    pub delay: u64,
}

/// Response from GET /traffic
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Traffic {
    pub up: u64,
    pub down: u64,
}

/// Response from GET /version
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VersionInfo {
    pub version: String,
    #[serde(default)]
    pub meta: Option<bool>,
}

/// Response from GET /configs
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ClashConfigs {
    pub mode: Option<String>,
    pub port: Option<u16>,
    #[serde(rename = "socks-port")]
    pub socks_port: Option<u16>,
    #[serde(rename = "mixed-port")]
    pub mixed_port: Option<u16>,
    #[serde(rename = "allow-lan")]
    pub allow_lan: Option<bool>,
    #[serde(rename = "log-level")]
    pub log_level: Option<String>,
    #[serde(default)]
    pub tun: Option<TunConfig>,
}

/// TUN configuration subset from /configs
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TunConfig {
    #[serde(default)]
    pub enable: Option<bool>,
}

/// Response from GET /proxies/{name}/delay
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LatencyResult {
    pub delay: u64,
}

/// Response from GET /connections
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConnectionsResponse {
    pub connections: Option<Vec<Connection>>,
    #[serde(default)]
    pub download_total: Option<u64>,
    #[serde(default)]
    pub upload_total: Option<u64>,
}

/// A single active connection
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    pub id: String,
    pub metadata: Option<ConnectionMetadata>,
    pub upload: Option<u64>,
    pub download: Option<u64>,
    pub start: Option<String>,
    pub chains: Option<Vec<String>>,
    pub rule: Option<String>,
    #[serde(rename = "rulePayload")]
    pub rule_payload: Option<String>,
    pub host: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionMetadata {
    pub network: Option<String>,
    #[serde(rename = "type")]
    pub meta_type: Option<String>,
    pub source_ip: Option<String>,
    pub source_port: Option<String>,
    pub destination_ip: Option<String>,
    pub destination_port: Option<String>,
    pub host: Option<String>,
    #[serde(rename = "dnsMode")]
    pub dns_mode: Option<String>,
    pub process: Option<String>,
}

/// A single log entry from GET /logs
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LogEntry {
    #[serde(rename = "type")]
    pub level: String,
    pub payload: Option<String>,
}

/// Response from GET /rules
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RulesResponse {
    pub rules: Option<Vec<Rule>>,
}

/// A single rule
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    #[serde(rename = "type")]
    pub rule_type: String,
    pub payload: Option<String>,
    pub proxy: Option<String>,
}
