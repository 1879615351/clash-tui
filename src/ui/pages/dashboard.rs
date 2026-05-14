use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph, Sparkline};
use ratatui::Frame;

use super::super::theme::Theme;
use super::{Page, PageId};
use crate::event::action::Action;
use crate::state::store::AppState;

pub struct DashboardPage;

impl DashboardPage {
    pub fn new() -> Self {
        Self
    }
}

impl Page for DashboardPage {
    fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme, state: &AppState) {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let top = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(34),
            ])
            .split(rows[0]);

        let bottom = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[1]);

        // Row 1: Core Status, Mode, Memory
        render_core_card(frame, top[0], theme, state);
        render_mode_card(frame, top[1], theme, state);
        render_memory_card(frame, top[2], theme, state);

        // Row 2: Upload traffic, Download traffic
        render_traffic_card(
            frame,
            bottom[0],
            theme,
            state,
            "Upload",
            state.upload_speed,
            state.total_upload,
        );
        render_traffic_card(
            frame,
            bottom[1],
            theme,
            state,
            "Download",
            state.download_speed,
            state.total_download,
        );
    }

    fn handle_key(&mut self, key: KeyEvent, _state: &AppState) -> Vec<Action> {
        match key.code {
            KeyCode::Char('1') => vec![Action::SetClashMode("rule".into())],
            KeyCode::Char('2') => vec![Action::SetClashMode("global".into())],
            KeyCode::Char('3') => vec![Action::SetClashMode("direct".into())],
            KeyCode::Char('m') => {
                // Cycle mode
                let next = match _state.clash_mode.as_str() {
                    "rule" => "global",
                    "global" => "direct",
                    _ => "rule",
                };
                vec![Action::SetClashMode(next.into())]
            }
            _ => vec![],
        }
    }

    fn page_id(&self) -> PageId {
        PageId::Dashboard
    }

    fn title(&self) -> &'static str {
        "Dashboard"
    }
}

fn render_core_card(frame: &mut Frame, area: Rect, theme: &Theme, state: &AppState) {
    let status = if state.core_running {
        Span::styled(
            "Running",
            Style::default()
                .fg(theme.success)
                .add_modifier(Modifier::BOLD),
        )
    } else if state.api_connected {
        Span::styled("Connected", Style::default().fg(theme.success))
    } else {
        Span::styled("Disconnected", Style::default().fg(theme.error))
    };

    let version = &state.core_version;
    let lines = vec![
        Line::from(vec![Span::raw("Status: "), status]),
        Line::from(vec![Span::raw(format!(
            "Version: {}",
            if version.is_empty() { "N/A" } else { version }
        ))]),
        Line::from(vec![Span::raw(format!(
            "Conns: {}",
            state.active_conn_count
        ))]),
    ];

    let para = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Core "))
        .style(Style::default().fg(theme.fg));
    frame.render_widget(para, area);
}

fn render_mode_card(frame: &mut Frame, area: Rect, theme: &Theme, state: &AppState) {
    let mode_style = match state.clash_mode.as_str() {
        "rule" => Style::default().fg(theme.info),
        "global" => Style::default().fg(theme.warning),
        "direct" => Style::default().fg(theme.success),
        _ => Style::default().fg(theme.fg),
    };

    let lines = vec![
        Line::from(vec![
            Span::raw("Mode: "),
            Span::styled(
                state.clash_mode.to_uppercase(),
                mode_style.add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::raw(" m  Cycle mode")]),
        Line::from(vec![Span::raw(" 1  Rule  2  Global  3  Direct")]),
    ];

    let para = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Mode "))
        .style(Style::default().fg(theme.fg));
    frame.render_widget(para, area);
}

fn render_memory_card(frame: &mut Frame, area: Rect, theme: &Theme, state: &AppState) {
    let mem_mb = state.memory_used_bytes / 1024 / 1024;
    let ratio = (mem_mb as f64 / 512.0).min(1.0); // assume 512MB as max

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" Memory "))
        .gauge_style(Style::default().fg(theme.accent))
        .ratio(ratio)
        .label(format!("{}MB", mem_mb));
    frame.render_widget(gauge, inner[1]);
}

fn render_traffic_card(
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    state: &AppState,
    label: &str,
    speed: u64,
    _total: u64,
) {
    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);

    let speed_label = format_speed(speed);

    let history = if label == "Upload" {
        &state.speed_history_up
    } else {
        &state.speed_history_down
    };

    // Convert to u64 values for sparkline (max 64 datapoints)
    let data: Vec<u64> = if history.is_empty() {
        vec![0]
    } else {
        history.iter().map(|&v| v / 1024).collect() // KB/s
    };

    let sparkline = Sparkline::default()
        .block(Block::default().title(format!(" {}: {}/s ", label, speed_label)))
        .data(&data)
        .style(Style::default().fg(theme.accent));

    frame.render_widget(sparkline, inner[1]);
}

fn format_speed(bytes: u64) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}
