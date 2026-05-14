use crossterm::event::KeyEvent;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Row, Table, TableState};
use ratatui::Frame;

use super::super::theme::Theme;
use super::{Page, PageId};
use ratatui::text::Span;

use crate::event::action::Action;
use crate::state::store::AppState;

pub struct RulesPage {
    selected: usize,
    table_state: TableState,
}

impl RulesPage {
    pub fn new() -> Self {
        Self {
            selected: 0,
            table_state: TableState::default(),
        }
    }
}

impl Page for RulesPage {
    fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme, state: &AppState) {
        let header = Row::new(vec!["Type", "Payload", "Proxy"]).style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        );

        let rows: Vec<Row> = state
            .rules
            .iter()
            .map(|rule| {
                let rule_type = &rule.rule_type;
                let payload = rule.payload.as_deref().unwrap_or("-");
                let proxy = rule.proxy.as_deref().unwrap_or("-");

                let type_style = match rule_type.as_str() {
                    "DOMAIN" => Style::default().fg(theme.info),
                    "DOMAIN-SUFFIX" => Style::default().fg(theme.success),
                    "DOMAIN-KEYWORD" => Style::default().fg(theme.warning),
                    "IP-CIDR" | "IP-CIDR6" => Style::default().fg(theme.accent),
                    "GEOIP" => Style::default().fg(theme.latency_mid),
                    "MATCH" => Style::default().fg(theme.error),
                    _ => Style::default().fg(theme.fg),
                };

                Row::new(vec![
                    Span::styled(rule_type, type_style).to_string(),
                    payload.to_string(),
                    proxy.to_string(),
                ])
            })
            .collect();

        let widths = [
            Constraint::Percentage(20),
            Constraint::Percentage(50),
            Constraint::Percentage(30),
        ];

        let title = format!(" Rules ({}) ", state.rules.len());
        let mut table_state = self.table_state.clone();

        let table = Table::new(rows, widths)
            .header(header)
            .block(Block::default().borders(Borders::ALL).title(title))
            .row_highlight_style(Style::default().fg(theme.surface).bg(theme.accent));

        frame.render_stateful_widget(table, area, &mut table_state);
    }

    fn handle_key(&mut self, key: KeyEvent, state: &AppState) -> Vec<Action> {
        use crossterm::event::KeyCode;
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                let max = state.rules.len().saturating_sub(1);
                self.selected = (self.selected + 1).min(max);
                vec![]
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected = self.selected.saturating_sub(1);
                vec![]
            }
            _ => vec![],
        }
    }

    fn page_id(&self) -> PageId {
        PageId::Rules
    }

    fn title(&self) -> &'static str {
        "Rules"
    }
}
