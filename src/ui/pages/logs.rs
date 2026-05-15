use std::cell::Cell;

use crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::Frame;

use super::super::theme::Theme;
use super::{Page, PageId};
use crate::event::action::Action;
use crate::state::store::AppState;

pub struct LogsPage {
    scroll: Cell<usize>,
    auto_scroll: Cell<bool>,
    prev_count: Cell<usize>,
    list_state: ListState,
    filter_level: Option<String>,
}

impl LogsPage {
    pub fn new() -> Self {
        Self {
            scroll: Cell::new(0),
            auto_scroll: Cell::new(true),
            prev_count: Cell::new(0),
            list_state: ListState::default(),
            filter_level: None,
        }
    }
}

impl Page for LogsPage {
    fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme, state: &AppState) {
        let filtered: Vec<_> = state
            .logs
            .iter()
            .filter(|entry| {
                if let Some(ref level) = self.filter_level {
                    entry.level.eq_ignore_ascii_case(level)
                } else {
                    true
                }
            })
            .collect();

        let count = filtered.len();
        let prev = self.prev_count.get();
        if count != prev || self.auto_scroll.get() {
            self.scroll.set(usize::MAX);
        }
        self.prev_count.set(count);

        let items: Vec<ListItem> = filtered
            .iter()
            .rev()
            .map(|entry| {
                let level_style = match entry.level.to_lowercase().as_str() {
                    "error" => Style::default()
                        .fg(theme.error)
                        .add_modifier(Modifier::BOLD),
                    "warning" | "warn" => Style::default().fg(theme.warning),
                    "info" => Style::default().fg(theme.info),
                    "debug" => Style::default().fg(theme.latency_offline),
                    _ => Style::default().fg(theme.fg),
                };
                let payload = entry.payload.as_deref().unwrap_or("");
                ListItem::new(Line::from(vec![
                    Span::styled(format!("[{}] ", entry.level.to_uppercase()), level_style),
                    Span::raw(payload),
                ]))
            })
            .collect();

        let auto = if self.auto_scroll.get() {
            "[auto]"
        } else {
            "[manual]"
        };
        let filter_info = if let Some(ref lvl) = self.filter_level {
            format!(" (filter: {}) ", lvl)
        } else {
            String::new()
        };
        let title = format!(
            " Mihomo Logs ({}){} {} (from mihomo.log) ",
            count, filter_info, auto
        );

        let mut list_state = self.list_state.clone();
        *list_state.offset_mut() = self.scroll.get();
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(title))
            .highlight_style(Style::default().fg(theme.surface).bg(theme.accent));

        frame.render_stateful_widget(list, area, &mut list_state);
    }

    fn handle_key(&mut self, key: KeyEvent, _state: &AppState) -> Vec<Action> {
        use crossterm::event::KeyCode;
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.scroll.set(self.scroll.get().saturating_add(1));
                vec![]
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let s = self.scroll.get().saturating_sub(1);
                self.scroll.set(s);
                self.auto_scroll.set(false);
                vec![]
            }
            KeyCode::Char('e') => {
                self.filter_level = Some("error".into());
                vec![]
            }
            KeyCode::Char('w') => {
                self.filter_level = Some("warning".into());
                vec![]
            }
            KeyCode::Char('i') => {
                self.filter_level = Some("info".into());
                vec![]
            }
            KeyCode::Char('a') => {
                self.filter_level = None;
                vec![]
            }
            KeyCode::Home => {
                self.scroll.set(0);
                self.auto_scroll.set(false);
                vec![]
            }
            KeyCode::End => {
                self.scroll.set(usize::MAX);
                self.auto_scroll.set(true);
                vec![]
            }
            _ => vec![],
        }
    }

    fn tick(&mut self) -> Vec<Action> {
        // Check if new logs arrived and auto-scroll
        // We can't access state in tick, so this is handled in the next render
        vec![]
    }

    fn page_id(&self) -> PageId {
        PageId::Logs
    }

    fn title(&self) -> &'static str {
        "Logs"
    }

    fn on_enter(&mut self) {
        self.auto_scroll.set(true);
        self.scroll.set(usize::MAX);
    }
}
