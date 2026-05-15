use std::collections::HashMap;

use async_trait::async_trait;

use super::types::*;
use super::{ClashApi, RefreshData};

/// HTTP-based Clash API client (Phase 1: direct mode)
#[derive(Clone)]
pub struct ClashClient {
    http: reqwest::Client,
    base_url: String,
    secret: Option<String>,
}

impl ClashClient {
    pub fn new(host: &str, port: u16, secret: Option<String>) -> Self {
        Self {
            http: reqwest::Client::builder()
                .no_proxy()
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            base_url: format!("http://{}:{}", host, port),
            secret,
        }
    }

    fn auth_header(&self) -> Option<String> {
        self.secret.as_ref().map(|s| format!("Bearer {}", s))
    }

    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> anyhow::Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.http.get(&url);
        if let Some(ref auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }
        let resp = req.send().await?;
        let status = resp.status();
        let body = resp.text().await?;
        if !status.is_success() {
            return Err(anyhow::anyhow!(
                "API returned {} for {}: {}",
                status.as_u16(),
                path,
                body.lines().take(1).collect::<String>()
            ));
        }
        let parsed = serde_json::from_str(&body)?;
        Ok(parsed)
    }

    async fn put<B: serde::Serialize + std::fmt::Debug>(
        &self,
        path: &str,
        body: &B,
    ) -> anyhow::Result<()> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.http.put(&url).json(body);
        if let Some(ref auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }
        req.send().await?;
        Ok(())
    }

    async fn patch<B: serde::Serialize + std::fmt::Debug>(
        &self,
        path: &str,
        body: &B,
    ) -> anyhow::Result<()> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.http.patch(&url).json(body);
        if let Some(ref auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }
        req.send().await?;
        Ok(())
    }

    async fn delete(&self, path: &str) -> anyhow::Result<()> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.http.delete(&url);
        if let Some(ref auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }
        req.send().await?;
        Ok(())
    }
}

#[async_trait]
impl ClashApi for ClashClient {
    async fn get_proxies(&self) -> anyhow::Result<HashMap<String, ProxyGroup>> {
        let resp: ProxiesResponse = self.get("/proxies").await?;
        Ok(resp.proxies)
    }

    async fn switch_proxy(&self, group: &str, proxy: &str) -> anyhow::Result<()> {
        let body = serde_json::json!({"name": proxy});
        let path = format!("/proxies/{}", encode_uri_component(group));
        self.put(&path, &body).await
    }

    async fn get_traffic(&self) -> anyhow::Result<Traffic> {
        self.get("/traffic").await
    }

    async fn get_memory(&self) -> anyhow::Result<u64> {
        let url = format!("{}/memory", self.base_url);
        let mut req = self.http.get(&url);
        if let Some(ref auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }
        let resp = req.send().await?;
        let text = resp.text().await?;
        let value: serde_json::Value = serde_json::from_str(&text)?;
        // /memory returns a bare number wrapped in various forms depending on version
        if let Some(n) = value.as_u64() {
            return Ok(n);
        }
        if let Some(obj) = value.as_object() {
            if let Some(n) = obj.get("memory").and_then(|v| v.as_u64()) {
                return Ok(n);
            }
        }
        Ok(0)
    }

    async fn get_version(&self) -> anyhow::Result<VersionInfo> {
        self.get("/version").await
    }

    async fn get_configs(&self) -> anyhow::Result<ClashConfigs> {
        self.get("/configs").await
    }

    async fn test_latency(
        &self,
        proxy: &str,
        url: &str,
        timeout: u16,
    ) -> anyhow::Result<LatencyResult> {
        let path = format!(
            "/proxies/{}/delay?url={}&timeout={}",
            encode_uri_component(proxy),
            encode_uri_component(url),
            timeout
        );
        self.get(&path).await
    }

    async fn get_connections(&self) -> anyhow::Result<Vec<Connection>> {
        let resp: ConnectionsResponse = self.get("/connections").await?;
        Ok(resp.connections.unwrap_or_default())
    }

    async fn close_connection(&self, id: &str) -> anyhow::Result<()> {
        let path = format!("/connections/{}", id);
        self.delete(&path).await
    }

    async fn close_all_connections(&self) -> anyhow::Result<()> {
        self.delete("/connections").await
    }

    async fn get_logs(&self) -> anyhow::Result<Vec<LogEntry>> {
        let url = format!("{}/logs", self.base_url);
        let mut req = self.http.get(&url);
        if let Some(ref auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }
        let resp = req.send().await?;
        let text = resp.text().await?;
        // /logs returns newline-delimited JSON objects
        let entries: Vec<LogEntry> = text
            .lines()
            .filter_map(|line| serde_json::from_str::<LogEntry>(line).ok())
            .collect();
        Ok(entries)
    }

    async fn get_rules(&self) -> anyhow::Result<Vec<Rule>> {
        let resp: RulesResponse = self.get("/rules").await?;
        Ok(resp.rules.unwrap_or_default())
    }

    /// Fetch key data points concurrently for periodic refresh.
    /// Individual endpoint failures are tolerated; defaults are used.
    async fn set_config_mode(&self, mode: &str) -> anyhow::Result<()> {
        let body = serde_json::json!({"mode": mode});
        self.patch("/configs", &body).await
    }

    async fn reload_config(&self, _config_path: &str) -> anyhow::Result<()> {
        // Mihomo restarts are handled externally by restart_mihomo().
        // This method is kept for API compatibility.
        Ok(())
    }

    async fn refresh_all(&self) -> anyhow::Result<RefreshData> {
        let (proxies, traffic, memory, configs, version, conns, logs, rules) = tokio::join!(
            self.get_proxies(),
            self.get_traffic(),
            self.get_memory(),
            self.get_configs(),
            self.get_version(),
            self.get_connections(),
            self.get_logs(),
            self.get_rules(),
        );

        let proxy_groups = proxies.unwrap_or_else(|e| {
            tracing::warn!("get_proxies failed: {}", e);
            Default::default()
        });
        let traffic = traffic.unwrap_or_else(|e| {
            tracing::warn!("get_traffic failed: {}", e);
            Traffic { up: 0, down: 0 }
        });
        let memory = memory.unwrap_or_else(|e| {
            tracing::warn!("get_memory failed: {}", e);
            0
        });
        let configs = configs.unwrap_or_else(|e| {
            tracing::warn!("get_configs failed: {}", e);
            ClashConfigs {
                mode: Some("rule".into()),
                port: None,
                socks_port: None,
                mixed_port: None,
                allow_lan: None,
                log_level: None,
            }
        });
        let version = version.unwrap_or_else(|e| {
            tracing::warn!("get_version failed: {}", e);
            VersionInfo {
                version: String::new(),
                meta: None,
            }
        });
        let connections = conns.unwrap_or_else(|e| {
            tracing::warn!("get_connections failed: {}", e);
            Vec::new()
        });
        let active_conn_count = connections.len();
        let logs = logs.unwrap_or_else(|e| {
            tracing::warn!("get_logs failed: {}", e);
            Vec::new()
        });
        let rules = rules.unwrap_or_else(|e| {
            tracing::warn!("get_rules failed: {}", e);
            Vec::new()
        });

        let mode = configs.mode.unwrap_or_else(|| "rule".into());

        Ok(RefreshData {
            proxy_groups,
            memory,
            mode,
            upload_speed: traffic.up,
            download_speed: traffic.down,
            active_conn_count,
            core_version: version.version,
            connections,
            logs,
            rules,
        })
    }
}

/// Minimal percent-encoding for proxy names in URL paths
fn encode_uri_component(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_uri_component() {
        assert_eq!(encode_uri_component("hello"), "hello");
        assert_eq!(encode_uri_component("hello world"), "hello%20world");
        assert_eq!(encode_uri_component("HK-01"), "HK-01");
    }
}
