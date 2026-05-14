use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use super::super::theme::Theme;
use crate::state::store::AppState;

pub fn draw(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let status = if state.api_connected {
        "Connected"
    } else {
        "Disconnected"
    };

    let right_text = format!(
        "{} | Mode: {} | Mem: {}MB | ↑ {} ↓ {} | Conns: {} ",
        status,
        state.clash_mode,
        state.memory_used_bytes / 1024 / 1024,
        format_bytes(state.upload_speed),
        format_bytes(state.download_speed),
        state.active_conn_count,
    );

    if let Some(ref msg) = state.status_line {
        let line = Line::from(vec![
            Span::styled(
                format!(" {} ", msg),
                Style::default()
                    .fg(theme.surface)
                    .bg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(right_text, Style::default().fg(theme.latency_offline)),
        ]);
        let para = Paragraph::new(line);
        frame.render_widget(para, area);
    } else {
        let para = Paragraph::new(right_text)
            .style(Style::default().fg(theme.fg).bg(theme.surface))
            .alignment(Alignment::Right);
        frame.render_widget(para, area);
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1}GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1}MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else {
        format!("{}B", bytes)
    }
}
