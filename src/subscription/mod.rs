/// TOML file wrapper for subscription list (TOML requires a top-level key).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct SubscriptionsFile {
    subscription: Vec<Subscription>,
}

/// A subscription profile.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Subscription {
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub last_updated: Option<String>,
}

/// Subscription manager handles downloading and storing Clash configs.
pub struct SubscriptionManager;

impl SubscriptionManager {
    /// Download a subscription (HTTP or Base64) and return the raw config.
    pub async fn download(url: &str) -> anyhow::Result<String> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        tracing::debug!("Downloading subscription: {}", url);
        let resp = client.get(url).send().await?;
        let body = resp.text().await?;

        // Try base64 decode
        if let Some(decoded) = base64_decode(&body) {
            if decoded.contains("proxies:") || decoded.contains("Proxy:") {
                tracing::debug!("Subscription decoded from base64");
                return Ok(decoded);
            }
        }

        // Already plain YAML/YAML-like
        Ok(body)
    }

    /// Parse a YAML Clash config into proxy entries.
    pub fn parse_config(yaml: &str) -> anyhow::Result<Vec<ProxyEntry>> {
        let doc: serde_yaml::Value = serde_yaml::from_str(yaml)?;
        let mut proxies = Vec::new();

        if let Some(proxy_list) = doc.get("proxies").and_then(|p| p.as_sequence()) {
            for proxy in proxy_list {
                if let Some(entry) = Self::parse_proxy(proxy) {
                    proxies.push(entry);
                }
            }
        }

        if let Some(proxy_list) = doc.get("Proxy").and_then(|p| p.as_sequence()) {
            for proxy in proxy_list {
                if let Some(entry) = Self::parse_proxy(proxy) {
                    proxies.push(entry);
                }
            }
        }

        Ok(proxies)
    }

    fn parse_proxy(proxy: &serde_yaml::Value) -> Option<ProxyEntry> {
        let name = proxy.get("name")?.as_str()?.to_string();
        let proxy_type = proxy
            .get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("unknown")
            .to_string();
        let server = proxy
            .get("server")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();
        let port = proxy.get("port").and_then(|p| p.as_u64()).unwrap_or(0) as u16;

        Some(ProxyEntry {
            name,
            proxy_type,
            server,
            port,
        })
    }

    /// Load subscription list from config file.
    pub fn load_subscriptions() -> anyhow::Result<Vec<Subscription>> {
        let path = Self::subscriptions_path()?;
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => match toml::from_str::<SubscriptionsFile>(&content) {
                    Ok(file) => {
                        tracing::info!("Loaded {} subscriptions", file.subscription.len());
                        Ok(file.subscription)
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse subscriptions: {}", e);
                        Ok(Vec::new())
                    }
                },
                Err(e) => {
                    tracing::warn!("Failed to read subscriptions: {}", e);
                    Ok(Vec::new())
                }
            }
        } else {
            Ok(Vec::new())
        }
    }

    /// Save subscription list to config file.
    pub fn save_subscriptions(subs: &[Subscription]) -> anyhow::Result<()> {
        let path = Self::subscriptions_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = SubscriptionsFile {
            subscription: subs.to_vec(),
        };
        let content = toml::to_string_pretty(&file)?;
        std::fs::write(&path, content)?;
        tracing::debug!("Saved {} subscriptions", subs.len());
        Ok(())
    }

    fn subscriptions_path() -> anyhow::Result<std::path::PathBuf> {
        crate::config::AppConfig::config_dir().map(|d| d.join("subscriptions.toml"))
    }
}

/// A parsed proxy entry from a subscription config.
#[derive(Debug, Clone)]
pub struct ProxyEntry {
    pub name: String,
    pub proxy_type: String,
    pub server: String,
    pub port: u16,
}

/// Try to base64-decode a string. Returns None if not valid base64.
fn base64_decode(input: &str) -> Option<String> {
    use base64::Engine;
    let cleaned = input.trim().replace(char::is_whitespace, "");
    base64::engine::general_purpose::STANDARD
        .decode(&cleaned)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_load_roundtrip() {
        let subs = vec![
            Subscription {
                name: "my-sub".into(),
                url: "https://example.com/sub".into(),
                enabled: true,
                last_updated: Some("12:00".into()),
            },
            Subscription {
                name: "another".into(),
                url: "https://example.com/other".into(),
                enabled: false,
                last_updated: None,
            },
        ];

        // Save
        SubscriptionManager::save_subscriptions(&subs).unwrap();

        // Load
        let loaded = SubscriptionManager::load_subscriptions().unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].name, "my-sub");
        assert_eq!(loaded[0].url, "https://example.com/sub");
        assert!(loaded[0].enabled);
        assert_eq!(loaded[0].last_updated, Some("12:00".into()));
        assert_eq!(loaded[1].name, "another");
        assert!(!loaded[1].enabled);
    }
}
