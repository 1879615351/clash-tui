use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Sparkline};
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
            .constraints([
                Constraint::Length(5),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(area);

        // Row 1: Core status + Mode + Proxy summary
        let top = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(30),
                Constraint::Percentage(30),
            ])
            .split(rows[0]);

        render_status_card(frame, top[0], theme, state);
        render_mode_card(frame, top[1], theme, state);
        render_proxy_summary(frame, top[2], theme, state);

        // Row 2: Upload + Download sparklines
        let mid = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[1]);

        render_traffic_sparkline(frame, mid[0], theme, state, "Upload");
        render_traffic_sparkline(frame, mid[1], theme, state, "Download");

        // Row 3: Memory gauge + quick tips
        let bottom = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[2]);

        render_memory_panel(frame, bottom[0], theme, state);
        render_tips_panel(frame, bottom[1], theme, state);
    }

    fn handle_key(&mut self, key: KeyEvent, _state: &AppState) -> Vec<Action> {
        match key.code {
            KeyCode::Char('1') => vec![Action::SetClashMode("rule".into())],
            KeyCode::Char('2') => vec![Action::SetClashMode("global".into())],
            KeyCode::Char('3') => vec![Action::SetClashMode("direct".into())],
            KeyCode::Char('R') => vec![Action::RestartMihomo],
            KeyCode::Char('m') => {
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

fn render_status_card(frame: &mut Frame, area: Rect, theme: &Theme, state: &AppState) {
    let (ver_style, ver_text) = if state.core_running {
        (
            Style::default()
                .fg(theme.success)
                .add_modifier(Modifier::BOLD),
            format!("mihomo {}", state.core_version),
        )
    } else if state.api_connected {
        (Style::default().fg(theme.success), "Connected".into())
    } else {
        (
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::BOLD),
            "Starting...".into(),
        )
    };

    let lines = vec![
        Line::from(vec![Span::styled(ver_text, ver_style)]),
        Line::from(vec![
            Span::raw("Mem: "),
            Span::styled(
                format_memory(state.memory_used_bytes),
                Style::default().fg(theme.accent),
            ),
            Span::raw("  Conns: "),
            Span::styled(
                format!("{}", state.active_conn_count),
                Style::default().fg(theme.info),
            ),
        ]),
        Line::from(vec![
            Span::raw("↑ "),
            Span::styled(
                format_speed(state.upload_speed),
                Style::default().fg(theme.latency_mid),
            ),
            Span::raw("  ↓ "),
            Span::styled(
                format_speed(state.download_speed),
                Style::default().fg(theme.accent),
            ),
        ]),
        Line::from(vec![Span::raw(format!(
            "{} groups  {} rules  {} logs",
            state.proxy_groups.len(),
            state.rules.len(),
            state.logs.len(),
        ))]),
    ];

    let para = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Status "))
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
        Line::from(vec![Span::raw(" m  cycle")]),
        Line::from(vec![Span::raw(" 1  Rule  2  Global  3  Direct")]),
    ];

    let para = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Mode "))
        .style(Style::default().fg(theme.fg));
    frame.render_widget(para, area);
}

fn render_proxy_summary(frame: &mut Frame, area: Rect, theme: &Theme, state: &AppState) {
    let mut groups: Vec<_> = state.proxy_groups.iter().collect();
    groups.sort_by(|a, b| a.0.cmp(b.0));

    let lines: Vec<Line> = groups
        .iter()
        .take(4)
        .map(|(name, group)| {
            let current = group.now.as_deref().unwrap_or("?");
            Line::from(vec![
                Span::styled(name.as_str(), Style::default().fg(theme.accent)),
                Span::raw(" → "),
                Span::raw(current),
            ])
        })
        .collect();

    let more = if groups.len() > 4 {
        format!("... and {} more", groups.len() - 4)
    } else {
        String::new()
    };

    let mut all_lines = lines;
    if !more.is_empty() {
        all_lines.push(Line::from(Span::styled(
            more,
            Style::default().fg(theme.latency_offline),
        )));
    }
    if all_lines.is_empty() {
        all_lines.push(Line::from("No proxies"));
    }

    let para = Paragraph::new(all_lines)
        .block(Block::default().borders(Borders::ALL).title(" Proxies "))
        .style(Style::default().fg(theme.fg));
    frame.render_widget(para, area);
}

fn render_traffic_sparkline(
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    state: &AppState,
    label: &str,
) {
    let (speed, history) = if label == "Upload" {
        (state.upload_speed, &state.speed_history_up)
    } else {
        (state.download_speed, &state.speed_history_down)
    };

    let data: Vec<u64> = if history.is_empty() {
        vec![0]
    } else {
        history.iter().map(|&v| v / 1024).collect()
    };

    let sparkline = Sparkline::default()
        .block(Block::default().title(format!(" {}: {}/s ", label, format_speed(speed))))
        .data(&data)
        .style(Style::default().fg(theme.accent));

    frame.render_widget(sparkline, area);
}

fn render_memory_panel(frame: &mut Frame, area: Rect, theme: &Theme, state: &AppState) {
    let lines = vec![
        Line::from(vec![
            Span::raw("Memory: "),
            Span::styled(
                format_memory(state.memory_used_bytes),
                Style::default().fg(theme.accent),
            ),
        ]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::styled(
            "API: 127.0.0.1:9090",
            Style::default().fg(theme.latency_offline),
        )]),
        Line::from(vec![Span::styled(
            "Proxy: 127.0.0.1:7890",
            Style::default().fg(theme.latency_offline),
        )]),
    ];

    let para = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Info "))
        .style(Style::default().fg(theme.fg));
    frame.render_widget(para, area);
}

fn render_tips_panel(frame: &mut Frame, area: Rect, theme: &Theme, _state: &AppState) {
    let lines = vec![
        Line::from(" Tab          Next page"),
        Line::from(" m             Cycle mode (Rule/Global/Direct)"),
        Line::from(" 1/2/3        Set mode"),
        Line::from(" R             Restart mihomo"),
        Line::from(" T             Cycle theme"),
    ];

    let para = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Keys "))
        .style(Style::default().fg(theme.latency_offline));
    frame.render_widget(para, area);
}

fn format_memory(bytes: u64) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.0} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
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
