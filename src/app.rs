use std::sync::Arc;

use tokio::sync::mpsc;

use crossterm::event::{self, Event, KeyEvent};
use ratatui::prelude::*;

use crate::clash::{ClashApi, RefreshData};
use crate::config::AppConfig;
use crate::event::action::Action;
use crate::event::keymap;
use crate::proxy::ProxyState;
use crate::state::store::AppState;
use crate::subscription::SubscriptionManager;
use crate::ui::components::{help_bar, status_bar, tabs};
use crate::ui::layout;
use crate::ui::pages::{PageId, PageRouter};
use crate::ui::theme::Theme;

pub struct App {
    pub state: AppState,
    pages: PageRouter,
    clash_client: Arc<dyn ClashApi>,
    config: AppConfig,
    theme: Theme,
    data_tx: mpsc::UnboundedSender<RefreshData>,
    data_rx: mpsc::UnboundedReceiver<RefreshData>,
    latency_tx: mpsc::UnboundedSender<(String, u64)>,
    latency_rx: mpsc::UnboundedReceiver<(String, u64)>,
    rt_handle: tokio::runtime::Handle,
    pub should_quit: bool,
}

impl App {
    pub fn with_client(
        config: AppConfig,
        theme: Theme,
        clash_client: Box<dyn ClashApi>,
        data_tx: mpsc::UnboundedSender<RefreshData>,
        data_rx: mpsc::UnboundedReceiver<RefreshData>,
        rt_handle: tokio::runtime::Handle,
    ) -> Self {
        let (latency_tx, latency_rx) = mpsc::unbounded_channel();
        let mut state = AppState::default();

        state.api_host = config.api.host.clone();
        state.api_port = config.api.port;

        if let Ok(proxy_state) = ProxyState::detect() {
            state.proxy_state = proxy_state;
        }

        if let Ok(subs) = SubscriptionManager::load_subscriptions() {
            state.subscriptions = subs;
        }

        Self {
            state,
            pages: PageRouter::new(),
            clash_client: Arc::from(clash_client),
            config,
            theme,
            data_tx,
            data_rx,
            latency_tx,
            latency_rx,
            rt_handle,
            should_quit: false,
        }
    }

    pub async fn run_async(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> anyhow::Result<()> {
        let client = self.clash_client.clone();
        let tx = self.data_tx.clone();
        let interval_ms = self.config.ui.refresh_interval_ms;

        // Background refresh loop — handles initial connection detection and periodic updates
        tokio::spawn(async move {
            refresh_loop(client.clone(), tx.clone(), interval_ms).await;
        });

        loop {
            terminal.draw(|f| self.render(f))?;

            if event::poll(std::time::Duration::from_millis(10))? {
                match event::read()? {
                    Event::Key(key) => {
                        if key.kind == crossterm::event::KeyEventKind::Press {
                            let actions = self.handle_input(key);
                            for action in actions {
                                self.dispatch(action);
                            }
                        }
                    }
                    Event::Resize(_, _) => {}
                    Event::Mouse(_) => {} // ignore mouse events
                    _ => {}
                }
            }

            if self.should_quit {
                break;
            }

            while let Ok(data) = self.data_rx.try_recv() {
                self.state.apply_refresh(data);
            }

            while let Ok((proxy, delay)) = self.latency_rx.try_recv() {
                self.state.latency_cache.insert(proxy, delay);
            }

            for action in self.pages.active_page_mut().tick() {
                self.dispatch(action);
            }

            self.state.tick_status();
        }

        Ok(())
    }

    fn handle_input(&mut self, key: KeyEvent) -> Vec<Action> {
        // If page is in a modal/input state, skip global keybindings
        if self.pages.active_page().is_modal() {
            return self.pages.active_page_mut().handle_key(key, &self.state);
        }
        let mut actions = keymap::map_key(key);
        let page_actions = self.pages.active_page_mut().handle_key(key, &self.state);
        actions.extend(page_actions);
        actions
    }

    fn dispatch(&mut self, action: Action) {
        match action {
            Action::Quit => self.should_quit = true,
            Action::NextPage => self.pages.next_page(),
            Action::PrevPage => self.pages.prev_page(),
            Action::SwitchPage(id) => self.pages.switch_to(id),
            Action::ToggleHelp => self.state.show_help = !self.state.show_help,

            Action::SelectProxy { group, proxy } => {
                // Optimistic UI update: immediately show the proxy as active
                if let Some(g) = self.state.proxy_groups.get_mut(&group) {
                    g.now = Some(proxy.clone());
                }
                self.state.set_status(format!("{} → {}", group, proxy));

                let client = self.clash_client.clone();
                let tx = self.data_tx.clone();
                self.rt_handle.spawn(async move {
                    if let Err(e) = client.switch_proxy(&group, &proxy).await {
                        tracing::error!("Switch proxy failed: {}", e);
                    }
                    if let Ok(data) = client.refresh_all().await {
                        let _ = tx.send(data);
                    }
                });
            }

            Action::TestLatency { group: _, proxy } => {
                let client = self.clash_client.clone();
                let lat_tx = self.latency_tx.clone();
                let p = proxy.clone();
                self.rt_handle.spawn(async move {
                    match client
                        .test_latency(&p, "https://www.gstatic.com/generate_204", 5000)
                        .await
                    {
                        Ok(result) => {
                            let _ = lat_tx.send((p, result.delay));
                        }
                        Err(e) => tracing::warn!("Latency failed: {}", e),
                    }
                });
            }

            Action::TestAllLatency { group: _, proxies } => {
                let client = self.clash_client.clone();
                let lat_tx = self.latency_tx.clone();
                let count = proxies.len();
                self.rt_handle.spawn(async move {
                    let test_url = "https://www.gstatic.com/generate_204";
                    for proxy in &proxies {
                        let p = proxy.clone();
                        let lt = lat_tx.clone();
                        let c = client.clone();
                        let url = test_url.to_string();
                        tokio::spawn(async move {
                            match c.test_latency(&p, &url, 3000).await {
                                Ok(result) => {
                                    let _ = lt.send((p, result.delay));
                                }
                                Err(_) => {}
                            }
                        });
                    }
                    tracing::debug!("Dispatched {} latency tests", count);
                });
            }

            Action::SetClashMode(mode) => {
                // Optimistic UI update
                self.state.clash_mode = mode.clone();
                self.state
                    .set_status(format!("Mode: {}", mode.to_uppercase()));
                let client = self.clash_client.clone();
                let tx = self.data_tx.clone();
                let m = mode.clone();
                self.rt_handle.spawn(async move {
                    if let Err(e) = client.set_config_mode(&m).await {
                        tracing::error!("Set mode failed: {}", e);
                    }
                    // Refresh to confirm
                    if let Ok(data) = client.refresh_all().await {
                        let _ = tx.send(data);
                    }
                });
            }
            Action::RestartMihomo => {
                let client = self.clash_client.clone();
                let tx = self.data_tx.clone();
                self.state.set_status("Restarting mihomo...");
                self.rt_handle.spawn(async move {
                    if let Err(e) = crate::core::CoreManager::restart_mihomo() {
                        tracing::error!("Restart failed: {}", e);
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    if let Ok(data) = client.refresh_all().await {
                        let _ = tx.send(data);
                    }
                });
            }
            Action::ToggleTun => {
                let enable = !self.state.tun_enabled;
                self.state.tun_enabled = enable;
                self.state
                    .set_status(format!("TUN: {}", if enable { "ON" } else { "OFF" }));
                let client = self.clash_client.clone();
                let tx = self.data_tx.clone();
                self.rt_handle.spawn(async move {
                    if let Err(e) = client.set_tun(enable).await {
                        tracing::error!("Set TUN failed: {}", e);
                    }
                    if let Ok(data) = client.refresh_all().await {
                        let _ = tx.send(data);
                    }
                });
            }
            Action::CloseConnection(id) => {
                let client = self.clash_client.clone();
                let tx = self.data_tx.clone();
                self.rt_handle.spawn(async move {
                    if let Err(e) = client.close_connection(&id).await {
                        tracing::error!("Close connection failed: {}", e);
                    }
                    if let Ok(data) = client.refresh_all().await {
                        let _ = tx.send(data);
                    }
                });
            }

            Action::CloseAllConnections => {
                let client = self.clash_client.clone();
                let tx = self.data_tx.clone();
                self.rt_handle.spawn(async move {
                    if let Err(e) = client.close_all_connections().await {
                        tracing::error!("Close all failed: {}", e);
                    }
                    if let Ok(data) = client.refresh_all().await {
                        let _ = tx.send(data);
                    }
                });
            }

            Action::ToggleSystemProxy => {
                if let Err(e) = self.state.proxy_state.toggle() {
                    tracing::error!("Proxy toggle failed: {}", e);
                } else {
                    self.state.set_status(format!(
                        "Proxy: {}",
                        if self.state.proxy_state.enabled {
                            "ON"
                        } else {
                            "OFF"
                        }
                    ));
                }
            }
            Action::EnableSystemProxy => {
                if let Err(e) = self.state.proxy_state.enable() {
                    tracing::error!("Proxy enable failed: {}", e);
                } else {
                    self.state.proxy_state.enabled = true;
                }
            }
            Action::DisableSystemProxy => {
                if let Err(e) = self.state.proxy_state.disable() {
                    tracing::error!("Proxy disable failed: {}", e);
                } else {
                    self.state.proxy_state.enabled = false;
                }
            }

            Action::DownloadSubscription(url) => {
                let url_c = url.clone();
                // Update last_updated timestamp
                let now = {
                    let t = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default();
                    let secs = t.as_secs();
                    let hours = (secs / 3600) % 24;
                    let mins = (secs / 60) % 60;
                    format!("{:02}:{:02}", hours, mins)
                };
                let mut sub_name = String::new();
                if let Some(sub) = self.state.subscriptions.iter_mut().find(|s| s.url == url_c) {
                    sub.last_updated = Some(now);
                    sub_name = sub.name.clone();
                }
                if let Err(e) = SubscriptionManager::save_subscriptions(&self.state.subscriptions) {
                    tracing::error!("Failed to save subscriptions: {}", e);
                }

                let client = self.clash_client.clone();
                let tx = self.data_tx.clone();
                let api_host = self.state.api_host.clone();
                let api_port = self.state.api_port;
                self.rt_handle.spawn(async move {
                    match SubscriptionManager::download(&url_c).await {
                        Ok(config) => {
                            let count = SubscriptionManager::parse_config(&config)
                                .map(|p| p.len())
                                .unwrap_or(0);
                            tracing::info!("Downloaded {} proxies for '{}'", count, sub_name);
                            match crate::core::CoreManager::save_subscription_config_with_api(
                                &sub_name, &config, &api_host, api_port,
                            ) {
                                Ok(config_path) => {
                                    tracing::info!("Config saved: {}", config_path);
                                    // Restart mihomo to load the new config
                                    if let Err(e) = crate::core::CoreManager::restart_mihomo() {
                                        tracing::warn!("Failed to restart mihomo: {}", e);
                                    }
                                    // Wait for mihomo to come back, then refresh
                                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                                    if let Ok(data) = client.refresh_all().await {
                                        let _ = tx.send(data);
                                    }
                                }
                                Err(e) => tracing::error!("Failed to save sub config: {}", e),
                            }
                        }
                        Err(e) => tracing::error!("Download failed: {}", e),
                    }
                });
            }

            Action::AddSubscription { name, url } => {
                let sub = crate::subscription::Subscription {
                    name,
                    url,
                    enabled: true,
                    last_updated: None,
                };
                self.state.subscriptions.push(sub);
                if let Err(e) = SubscriptionManager::save_subscriptions(&self.state.subscriptions) {
                    tracing::error!("Failed to save subscriptions: {}", e);
                }
            }
            Action::RemoveSubscription(name) => {
                self.state.subscriptions.retain(|s| s.name != name);
                if let Err(e) = SubscriptionManager::save_subscriptions(&self.state.subscriptions) {
                    tracing::error!("Failed to save subscriptions: {}", e);
                }
            }
            Action::ToggleSubscription(name) => {
                if let Some(sub) = self.state.subscriptions.iter_mut().find(|s| s.name == name) {
                    sub.enabled = !sub.enabled;
                    if let Err(e) =
                        SubscriptionManager::save_subscriptions(&self.state.subscriptions)
                    {
                        tracing::error!("Failed to save subscriptions: {}", e);
                    }
                }
            }

            Action::RefreshData => {
                let client = self.clash_client.clone();
                let tx = self.data_tx.clone();
                self.rt_handle.spawn(async move {
                    if let Ok(data) = client.refresh_all().await {
                        let _ = tx.send(data);
                    }
                });
            }

            Action::UpdateData(data) => self.state.apply_refresh(data),
            Action::CycleTheme => {
                let themes = ["tokyo-night", "catppuccin", "gruvbox"];
                let current = self.config.ui.theme.as_str();
                let next = themes
                    .iter()
                    .position(|t| *t == current)
                    .map(|i| themes[(i + 1) % themes.len()])
                    .unwrap_or(themes[0]);
                if let Ok(new_theme) = Theme::load(next) {
                    self.theme = new_theme;
                    self.config.ui.theme = next.to_string();
                    let _ = self.config.save();
                    self.state.set_status(format!("Theme: {}", next));
                }
            }
            Action::Error(msg) => tracing::warn!("{}", msg),
            Action::Noop | Action::SelectNext | Action::SelectPrev => {}
        }
    }

    fn render(&self, frame: &mut Frame) {
        let area = frame.area();
        let (tab_area, content_area, status_area) = layout::app_layout(area);

        tabs::draw(
            frame,
            tab_area,
            PageId::all(),
            self.pages.active_id(),
            &self.theme,
        );
        self.pages
            .active_page()
            .render(frame, content_area, &self.theme, &self.state);
        status_bar::draw(frame, status_area, &self.state, &self.theme);

        if self.state.show_help {
            help_bar::draw(frame, area, &self.theme, self.pages.active_id());
        }
    }
}

async fn refresh_loop(
    client: Arc<dyn ClashApi>,
    tx: mpsc::UnboundedSender<RefreshData>,
    interval_ms: u64,
) {
    // Fire immediately, then at interval
    let mut first = true;
    let mut interval = tokio::time::interval(std::time::Duration::from_millis(interval_ms));
    let mut tick_count = 0u64;
    tracing::info!("Refresh loop started ({}ms interval)", interval_ms);
    loop {
        if first {
            first = false;
        } else {
            interval.tick().await;
        }
        tick_count += 1;
        match client.refresh_all().await {
            Ok(data) => {
                if tick_count % 10 == 0 {
                    tracing::info!(
                        "Refresh OK: {} proxies, {}MB mem, {}B/s up, {} conns",
                        data.proxy_groups.len(),
                        data.memory / 1024 / 1024,
                        data.upload_speed,
                        data.active_conn_count,
                    );
                }
                if tx.send(data).is_err() {
                    break;
                }
            }
            Err(e) => {
                tracing::warn!("Refresh loop error (tick {}): {}", tick_count, e);
            }
        }
    }
}
