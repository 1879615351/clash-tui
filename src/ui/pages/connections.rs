use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table, TableState};
use ratatui::Frame;

use super::super::theme::Theme;
use super::{Page, PageId};
use crate::event::action::Action;
use crate::state::store::AppState;

pub struct ConnectionsPage {
    selected: usize,
    table_state: TableState,
}

impl ConnectionsPage {
    pub fn new() -> Self {
        Self {
            selected: 0,
            table_state: TableState::default(),
        }
    }
}

impl Page for ConnectionsPage {
    fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme, state: &AppState) {
        let has_conns = !state.connections.is_empty();
        let (table_area, detail_area) = if has_conns {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
                .split(area);
            (chunks[0], chunks[1])
        } else {
            (area, Rect::default())
        };

        // Table
        let header = Row::new(vec!["Host", "Network", "Rule", "Upload", "Download"]).style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        );

        let rows: Vec<Row> = state
            .connections
            .iter()
            .map(|conn| {
                let host = conn
                    .metadata
                    .as_ref()
                    .and_then(|m| m.host.as_deref())
                    .or(conn.host.as_deref())
                    .unwrap_or("-");
                let network = conn
                    .metadata
                    .as_ref()
                    .and_then(|m| m.network.as_deref())
                    .unwrap_or("-");
                let rule = conn.rule.as_deref().unwrap_or("-");
                let upload = format_bytes(conn.upload.unwrap_or(0));
                let download = format_bytes(conn.download.unwrap_or(0));

                Row::new(vec![
                    host.to_string(),
                    network.to_string(),
                    rule.to_string(),
                    upload,
                    download,
                ])
            })
            .collect();

        let widths = [
            Constraint::Percentage(30),
            Constraint::Percentage(12),
            Constraint::Percentage(18),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ];

        let title = format!(" Connections ({}) ", state.connections.len());
        let mut table_state = self.table_state.clone();

        let table = Table::new(rows, widths)
            .header(header)
            .block(Block::default().borders(Borders::ALL).title(title))
            .row_highlight_style(
                Style::default()
                    .fg(theme.surface)
                    .bg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_stateful_widget(table, table_area, &mut table_state);

        // Detail panel
        if let Some(conn) = state.connections.get(self.selected) {
            let meta = conn.metadata.as_ref();
            let lines = vec![
                Line::from(vec![
                    Span::styled(" Host: ", Style::default().fg(theme.accent)),
                    Span::raw(meta.and_then(|m| m.host.as_deref()).unwrap_or("-")),
                ]),
                Line::from(vec![
                    Span::styled(" Dest: ", Style::default().fg(theme.accent)),
                    Span::raw(format!(
                        "{}:{}",
                        meta.and_then(|m| m.destination_ip.as_deref())
                            .unwrap_or("-"),
                        meta.and_then(|m| m.destination_port.as_deref())
                            .unwrap_or("-")
                    )),
                ]),
                Line::from(vec![
                    Span::styled(" Source: ", Style::default().fg(theme.accent)),
                    Span::raw(format!(
                        "{}:{}",
                        meta.and_then(|m| m.source_ip.as_deref()).unwrap_or("-"),
                        meta.and_then(|m| m.source_port.as_deref()).unwrap_or("-")
                    )),
                ]),
                Line::from(vec![
                    Span::styled(" Network: ", Style::default().fg(theme.accent)),
                    Span::raw(meta.and_then(|m| m.network.as_deref()).unwrap_or("-")),
                ]),
                Line::from(vec![
                    Span::styled(" Rule: ", Style::default().fg(theme.accent)),
                    Span::raw(conn.rule.as_deref().unwrap_or("-")),
                ]),
                Line::from(vec![
                    Span::styled(" Process: ", Style::default().fg(theme.accent)),
                    Span::raw(meta.and_then(|m| m.process.as_deref()).unwrap_or("-")),
                ]),
                Line::from(vec![
                    Span::styled(" Upload: ", Style::default().fg(theme.accent)),
                    Span::raw(format_bytes(conn.upload.unwrap_or(0))),
                    Span::raw("  "),
                    Span::styled(" Download: ", Style::default().fg(theme.accent)),
                    Span::raw(format_bytes(conn.download.unwrap_or(0))),
                ]),
                Line::from(vec![Span::raw("")]),
                Line::from(vec![Span::raw(" d  Close this connection")]),
                Line::from(vec![Span::raw(" D  Close all connections")]),
            ];
            let para = Paragraph::new(lines)
                .block(Block::default().borders(Borders::ALL).title(" Detail "))
                .style(Style::default().fg(theme.fg));
            frame.render_widget(para, detail_area);
        }
    }

    fn handle_key(&mut self, key: KeyEvent, state: &AppState) -> Vec<Action> {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                let max = state.connections.len().saturating_sub(1);
                self.selected = (self.selected + 1).min(max);
                vec![]
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected = self.selected.saturating_sub(1);
                vec![]
            }
            KeyCode::Char('d') => {
                if let Some(conn) = state.connections.get(self.selected) {
                    return vec![Action::CloseConnection(conn.id.clone())];
                }
                vec![]
            }
            KeyCode::Char('D') => {
                if !state.connections.is_empty() {
                    return vec![Action::CloseAllConnections];
                }
                vec![]
            }
            _ => vec![],
        }
    }

    fn page_id(&self) -> PageId {
        PageId::Connections
    }

    fn title(&self) -> &'static str {
        "Connections"
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1}MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else {
        format!("{}B", bytes)
    }
}
