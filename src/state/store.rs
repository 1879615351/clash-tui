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
    pub proxy_state: ProxyState,
    pub subscriptions: Vec<Subscription>,
    pub speed_history_up: Vec<u64>,
    pub speed_history_down: Vec<u64>,
    pub status_line: Option<String>,
    status_ttl: u8,
    pub active_page: PageId,
    pub show_help: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            api_connected: false,
            core_version: String::new(),
            core_running: false,
            clash_mode: "rule".into(),
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
            proxy_state: ProxyState::default(),
            subscriptions: Vec::new(),
            speed_history_up: Vec::new(),
            speed_history_down: Vec::new(),
            status_line: None,
            status_ttl: 0,
            active_page: PageId::Dashboard,
            show_help: false,
        }
    }
}

impl AppState {
    pub fn apply_refresh(&mut self, data: RefreshData) {
        self.proxy_groups = data.proxy_groups;
        self.memory_used_bytes = data.memory;
        self.clash_mode = data.mode;
        self.upload_speed = data.upload_speed;
        self.download_speed = data.download_speed;
        self.active_conn_count = data.active_conn_count;
        self.core_version = data.core_version;
        self.connections = data.connections;
        self.logs = data.logs;
        self.rules = data.rules;
        self.core_running = true;
        self.api_connected = true;

        // Track speed history (max 60 samples ≈ 1 minute)
        const MAX_SAMPLES: usize = 60;
        self.speed_history_up.push(data.upload_speed);
        self.speed_history_down.push(data.download_speed);
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

    pub fn set_disconnected(&mut self) {
        self.api_connected = false;
        self.core_running = false;
    }
}
