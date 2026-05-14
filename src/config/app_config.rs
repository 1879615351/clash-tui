use serde::{Deserialize, Serialize};

const DEFAULT_API_HOST: &str = "127.0.0.1";
const DEFAULT_API_PORT: u16 = 9090;
const DEFAULT_THEME: &str = "tokyo-night";
const DEFAULT_REFRESH_MS: u64 = 1000;
const DEFAULT_SUB_INTERVAL_HOURS: u32 = 24;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    #[serde(default)]
    pub core: CoreConfig,
    #[serde(default)]
    pub api: ApiConfig,
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub hotkey: HotkeyConfig,
    #[serde(default)]
    pub subscription: SubscriptionConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CoreConfig {
    #[serde(default = "default_core_type")]
    pub core_type: String,
    #[serde(default = "default_core_path")]
    pub core_path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiConfig {
    #[serde(default = "default_api_host")]
    pub host: String,
    #[serde(default = "default_api_port")]
    pub port: u16,
    #[serde(default)]
    pub secret: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UiConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_refresh_ms")]
    pub refresh_interval_ms: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HotkeyConfig {
    #[serde(default)]
    pub toggle: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionConfig {
    #[serde(default)]
    pub auto_update: bool,
    #[serde(default = "default_sub_interval")]
    pub interval_hours: u32,
}

fn default_core_type() -> String {
    "mihomo".into()
}

fn default_core_path() -> String {
    "./core/mihomo".into()
}

fn default_api_host() -> String {
    DEFAULT_API_HOST.into()
}

fn default_api_port() -> u16 {
    DEFAULT_API_PORT
}

fn default_theme() -> String {
    DEFAULT_THEME.into()
}

fn default_refresh_ms() -> u64 {
    DEFAULT_REFRESH_MS
}

fn default_sub_interval() -> u32 {
    DEFAULT_SUB_INTERVAL_HOURS
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            core: CoreConfig::default(),
            api: ApiConfig::default(),
            ui: UiConfig::default(),
            hotkey: HotkeyConfig::default(),
            subscription: SubscriptionConfig::default(),
        }
    }
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            core_type: default_core_type(),
            core_path: default_core_path(),
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            host: default_api_host(),
            port: default_api_port(),
            secret: None,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            refresh_interval_ms: default_refresh_ms(),
        }
    }
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self { toggle: None }
    }
}

impl Default for SubscriptionConfig {
    fn default() -> Self {
        Self {
            auto_update: false,
            interval_hours: default_sub_interval(),
        }
    }
}

impl AppConfig {
    /// Load config from the standard user config directory.
    /// Falls back to defaults if no config file exists.
    pub fn load() -> anyhow::Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: AppConfig = toml::from_str(&content)?;
            tracing::info!("Loaded config from {}", config_path.display());
            Ok(config)
        } else {
            let config = AppConfig::default();
            config.save()?;
            tracing::info!("Created default config at {}", config_path.display());
            Ok(config)
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let config_path = Self::config_path()?;
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }

    pub fn config_dir() -> anyhow::Result<std::path::PathBuf> {
        let dir = if cfg!(windows) {
            std::env::var("APPDATA")
                .map(|p| std::path::PathBuf::from(p).join("clash-tui"))
                .unwrap_or_else(|_| dirs_fallback())
        } else {
            dirs_fallback()
        };
        Ok(dir)
    }

    pub fn config_path() -> anyhow::Result<std::path::PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    pub fn themes_dir() -> anyhow::Result<std::path::PathBuf> {
        Ok(Self::config_dir()?.join("themes"))
    }
}

fn dirs_fallback() -> std::path::PathBuf {
    let proj = directories::ProjectDirs::from("com", "clash-tui-rs", "clash-tui")
        .unwrap_or_else(|| panic!("Failed to determine config directory"));
    proj.config_dir().to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_roundtrip() {
        let config = AppConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: AppConfig = toml::from_str(&toml_str).unwrap();

        assert_eq!(parsed.api.host, "127.0.0.1");
        assert_eq!(parsed.api.port, 9090);
        assert_eq!(parsed.ui.theme, "tokyo-night");
        assert_eq!(parsed.ui.refresh_interval_ms, 1000);
        assert_eq!(parsed.core.core_type, "mihomo");
    }

    #[test]
    fn test_config_with_secret() {
        let toml_str = r#"
[api]
host = "192.168.1.1"
port = 9091
secret = "my-secret-token"

[ui]
theme = "catppuccin"
"#;
        let config: AppConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.api.host, "192.168.1.1");
        assert_eq!(config.api.port, 9091);
        assert_eq!(config.api.secret, Some("my-secret-token".into()));
        assert_eq!(config.ui.theme, "catppuccin");
    }
}
