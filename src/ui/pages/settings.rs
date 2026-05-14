use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use super::super::theme::Theme;
use super::{Page, PageId};
use crate::event::action::Action;
use crate::state::store::AppState;

pub struct SettingsPage;

impl SettingsPage {
    pub fn new() -> Self {
        Self
    }
}

impl Page for SettingsPage {
    fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme, state: &AppState) {
        let proxy_status = if state.proxy_state.enabled {
            Span::styled(
                "ENABLED",
                Style::default()
                    .fg(theme.success)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled("DISABLED", Style::default().fg(theme.latency_offline))
        };

        let proxy_addr = state.proxy_state.address();

        let lines = vec![
            Line::from(vec![Span::styled(
                " System Proxy ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::raw("  Status: "), proxy_status]),
            Line::from(vec![Span::raw(format!("  Address: {}", proxy_addr))]),
            Line::from(vec![Span::raw("  p         Toggle system proxy on/off")]),
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::styled(
                " Connection ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::raw(format!(
                "  API: {}:{}",
                state.api_host, state.api_port
            ))]),
            Line::from(vec![Span::raw(format!(
                "  Config: {}",
                crate::config::AppConfig::config_path()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|_| "N/A".into())
            ))]),
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::styled(
                " Core ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::raw(format!(
                "  Version: {}",
                if state.core_version.is_empty() {
                    "N/A"
                } else {
                    &state.core_version
                }
            ))]),
            Line::from(vec![Span::raw(format!(
                "  Status: {}",
                if state.core_running {
                    "Running"
                } else {
                    "Disconnected"
                }
            ))]),
            Line::from(vec![Span::raw(format!(
                "  Mode: {}",
                state.clash_mode.to_uppercase()
            ))]),
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::styled(
                " Traffic ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::raw(format!(
                "  Memory: {}MB | Upload: {} | Download: {} | Conns: {}",
                state.memory_used_bytes / 1024 / 1024,
                format_speed(state.upload_speed),
                format_speed(state.download_speed),
                state.active_conn_count
            ))]),
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::styled(
                " Data ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::raw(format!(
                "  {} groups | {} rules | {} logs | {} connections",
                state.proxy_groups.len(),
                state.rules.len(),
                state.logs.len(),
                state.connections.len()
            ))]),
        ];

        let para = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title(" Settings "))
            .style(Style::default().fg(theme.fg));

        frame.render_widget(para, area);
    }

    fn handle_key(&mut self, key: KeyEvent, _state: &AppState) -> Vec<Action> {
        match key.code {
            KeyCode::Char('p') => vec![Action::ToggleSystemProxy],
            KeyCode::Char('P') => vec![Action::EnableSystemProxy],
            KeyCode::Char('o') => vec![Action::DisableSystemProxy],
            KeyCode::Char('T') => vec![Action::CycleTheme],
            _ => vec![],
        }
    }

    fn page_id(&self) -> PageId {
        PageId::Settings
    }

    fn title(&self) -> &'static str {
        "Settings"
    }
}

fn format_speed(bytes: u64) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1} MB/s", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB/s", bytes as f64 / 1024.0)
    } else {
        format!("{} B/s", bytes)
    }
}
