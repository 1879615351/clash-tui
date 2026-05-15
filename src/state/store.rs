use std::collections::HashMap;

use crate::clash::types::{Connection, LogEntry, ProxyGroup, Rule};
use crate::clash::RefreshData;
use crate::proxy::ProxyState;
use crate::subscription::Subscription;
use crate::ui::pages::PageId;

/// Central application state
#[derive(Debug, Clone)]
pub struct AppState {
    pub api_connected: bool,
    pub core_version: String,
    pub core_running: bool,
    pub clash_mode: String,
    pub tun_enabled: bool,
    pub memory_used_bytes: u64,
    pub upload_speed: u64,
    pub download_speed: u64,
    pub total_upload: u64,
    pub total_download: u64,
    pub active_conn_count: usize,
    pub proxy_groups: HashMap<String, ProxyGroup>,
    pub connections: Vec<Connection>,
    pub logs: Vec<LogEntry>,
    pub rules: Vec<Rule>,
    pub api_host: String,
    pub api_port: u16,
    /// Cached latency results (proxy_name → delay_ms), used because mihomo
    /// doesn't persist /delay results into proxy history.
    pub latency_cache: std::collections::HashMap<String, u64>,
    prev_traffic_up: u64,
    prev_traffic_down: u64,
    pub proxy_state: ProxyState,
    pub subscriptions: Vec<Subscription>,
    pub speed_history_up: Vec<u64>,
    pub speed_history_down: Vec<u64>,
    pub status_line: Option<String>,
    status_ttl: u8,
    pub active_page: PageId,
    pub show_help: bool,
    consecutive_failures: u8,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            api_connected: false,
            core_version: String::new(),
            core_running: false,
            clash_mode: "rule".into(),
            tun_enabled: false,
            memory_used_bytes: 0,
            upload_speed: 0,
            download_speed: 0,
            total_upload: 0,
            total_download: 0,
            active_conn_count: 0,
            proxy_groups: HashMap::new(),
            connections: Vec::new(),
            logs: Vec::new(),
            rules: Vec::new(),
            api_host: "127.0.0.1".into(),
            api_port: 9090,
            latency_cache: std::collections::HashMap::new(),
            prev_traffic_up: 0,
            prev_traffic_down: 0,
            proxy_state: ProxyState::default(),
            subscriptions: Vec::new(),
            speed_history_up: Vec::new(),
            speed_history_down: Vec::new(),
            status_line: None,
            status_ttl: 0,
            active_page: PageId::Dashboard,
            show_help: false,
            consecutive_failures: 0,
        }
    }
}

impl AppState {
    pub fn apply_refresh(&mut self, data: RefreshData) {
        self.proxy_groups = data.proxy_groups;
        self.memory_used_bytes = data.memory;
        self.clash_mode = data.mode;
        self.tun_enabled = data.tun_enabled;
        // /traffic returns cumulative bytes — compute speed as delta
        if self.prev_traffic_up == 0 {
            self.upload_speed = 0;
            self.download_speed = 0;
        } else {
            self.upload_speed = data.upload_speed.saturating_sub(self.prev_traffic_up);
            self.download_speed = data.download_speed.saturating_sub(self.prev_traffic_down);
        }
        self.prev_traffic_up = data.upload_speed;
        self.prev_traffic_down = data.download_speed;
        self.total_upload = data.upload_speed;
        self.total_download = data.download_speed;
        self.active_conn_count = data.active_conn_count;
        // Only overwrite version with non-empty data (preserve last known good)
        if !data.core_version.is_empty() {
            self.core_version = data.core_version;
        }
        self.connections = data.connections;
        // Prefer API-delivered logs (real-time, unbuffered); fall back to file
        if !data.logs.is_empty() {
            self.logs = data.logs;
        } else {
            self.load_mihomo_logs();
        }
        self.rules = data.rules;

        // Connection state: latch upward immediately, debounce downward
        // so a stale tick (started before mihomo was ready) can't revert
        // a fresh "connected" signal.
        if data.api_reachable {
            self.api_connected = true;
            self.core_running = true;
            self.consecutive_failures = 0;
        } else {
            self.consecutive_failures += 1;
            if self.consecutive_failures >= 3 {
                self.api_connected = false;
                self.core_running = false;
            }
        }

        // Track speed history (max 60 samples ≈ 1 minute)
        const MAX_SAMPLES: usize = 60;
        self.speed_history_up.push(self.upload_speed);
        self.speed_history_down.push(self.download_speed);
        if self.speed_history_up.len() > MAX_SAMPLES {
            self.speed_history_up.remove(0);
            self.speed_history_down.remove(0);
        }
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_line = Some(msg.into());
        self.status_ttl = 40; // ~4s
    }

    pub fn tick_status(&mut self) {
        if self.status_ttl > 0 {
            self.status_ttl -= 1;
            if self.status_ttl == 0 {
                self.status_line = None;
            }
        }
    }

    pub fn subscriptions_path() -> String {
        crate::config::AppConfig::config_dir()
            .map(|d| d.join("subscriptions.toml").display().to_string())
            .unwrap_or_else(|_| "~/.config/clash-tui/subscriptions.toml".into())
    }

    /// Read mihomo's log file and parse into LogEntry list.
    fn load_mihomo_logs(&mut self) {
        if let Ok(core_dir) =
            crate::config::AppConfig::config_dir().map(|d| d.join("core").join("mihomo.log"))
        {
            if let Ok(content) = std::fs::read_to_string(&core_dir) {
                self.logs = content
                    .lines()
                    .filter(|l| !l.is_empty())
                    .map(|line| LogEntry {
                        level: if line.contains("ERROR") || line.contains("error") {
                            "error".into()
                        } else if line.contains("WARN") || line.contains("warn") {
                            "warning".into()
                        } else if line.contains("DEBUG") || line.contains("debug") {
                            "debug".into()
                        } else {
                            "info".into()
                        },
                        payload: Some(line.to_string()),
                    })
                    .collect();
            }
        }
    }

    pub fn set_disconnected(&mut self) {
        self.api_connected = false;
        self.core_running = false;
    }
}
