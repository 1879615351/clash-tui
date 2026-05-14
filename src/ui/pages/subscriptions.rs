use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use super::super::theme::Theme;
use super::{Page, PageId};
use crate::event::action::Action;
use crate::state::store::AppState;

enum InputMode {
    None,
    AddUrl,
}

pub struct SubscriptionsPage {
    selected: usize,
    list_state: ListState,
    input_mode: InputMode,
    input_text: String,
    cursor_pos: usize,
    scroll_offset: usize,
}

impl SubscriptionsPage {
    pub fn new() -> Self {
        Self {
            selected: 0,
            list_state: ListState::default(),
            input_mode: InputMode::None,
            input_text: String::new(),
            cursor_pos: 0,
            scroll_offset: 0,
        }
    }

    /// Render the visible portion of the input text with cursor and scroll.
    fn render_input_text(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let prefix = " URL: ";
        let prefix_len = prefix.len();
        let inner_width = area.width.saturating_sub(4) as usize; // borders

        // Ensure cursor is visible by adjusting scroll
        let display_width = inner_width.saturating_sub(prefix_len);
        if display_width == 0 {
            return;
        }

        // Clamp cursor
        let cursor = self.cursor_pos.min(self.input_text.len());

        // Adjust scroll to keep cursor visible
        let mut scroll = self.scroll_offset;
        if cursor < scroll {
            scroll = cursor;
        }
        let end = if self.input_text.len() <= scroll + display_width {
            // Text fits entirely
            scroll = 0;
            self.input_text.len()
        } else {
            // Need scrolling
            if cursor >= scroll + display_width {
                scroll = cursor.saturating_sub(display_width) + 1;
            }
            (scroll + display_width).min(self.input_text.len())
        };

        // Build display line
        let visible = &self.input_text[scroll..end];
        let cursor_col = prefix_len + (cursor - scroll);

        let has_left = scroll > 0;
        let has_right = end < self.input_text.len();

        let left_marker = if has_left { "←" } else { " " };
        let right_marker = if has_right { "→" } else { " " };

        let full = format!(
            "{}{}{}{}",
            left_marker,
            prefix.trim_start(),
            visible,
            right_marker
        );

        let display_line = if cursor >= self.input_text.len() {
            // Cursor at end
            format!("{}█", full)
        } else {
            // Insert cursor character
            let mut chars: Vec<char> = full.chars().collect();
            if cursor_col < chars.len() {
                chars.insert(cursor_col, '▌');
            }
            chars.into_iter().collect::<String>()
        };

        let input_block = Block::default()
            .borders(Borders::ALL)
            .title(" Add Subscription (Enter: confirm, Esc: cancel, ←→: move cursor) ");

        let input_para = Paragraph::new(display_line)
            .block(input_block)
            .style(Style::default().fg(theme.accent));

        frame.render_widget(input_para, area);
    }
}

impl Page for SubscriptionsPage {
    fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme, state: &AppState) {
        let main_area = match self.input_mode {
            InputMode::None => area,
            _ => {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(0), Constraint::Length(3)])
                    .split(area);
                self.render_input_text(frame, chunks[1], theme);
                chunks[0]
            }
        };

        let panels = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(main_area);
        let left = panels[0];
        let right = panels[1];

        let items: Vec<ListItem> = state
            .subscriptions
            .iter()
            .enumerate()
            .map(|(i, sub)| {
                let enabled = if sub.enabled { "✓" } else { "✗" };
                let updated = sub.last_updated.as_deref().unwrap_or("never");
                let style = if i == self.selected {
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.fg)
                };
                ListItem::new(Line::from(vec![
                    Span::styled(format!("{} {} ", enabled, sub.name), style),
                    Span::styled(
                        format!("({})", updated),
                        Style::default().fg(theme.latency_offline),
                    ),
                ]))
            })
            .collect();

        let title = format!(" Subscriptions ({}) ", state.subscriptions.len());
        let mut list_state = self.list_state.clone();
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(title))
            .highlight_style(Style::default().fg(theme.surface).bg(theme.accent));

        frame.render_stateful_widget(list, left, &mut list_state);

        let detail_lines = if let Some(sub) = state.subscriptions.get(self.selected) {
            vec![
                Line::from(vec![
                    Span::styled(" Name: ", Style::default().fg(theme.accent)),
                    Span::raw(&sub.name),
                ]),
                Line::from(vec![
                    Span::styled(" URL: ", Style::default().fg(theme.accent)),
                    Span::raw(&sub.url),
                ]),
                Line::from(vec![
                    Span::styled(" Enabled: ", Style::default().fg(theme.accent)),
                    Span::raw(if sub.enabled { "Yes" } else { "No" }),
                ]),
                Line::from(vec![
                    Span::styled(" Updated: ", Style::default().fg(theme.accent)),
                    Span::raw(sub.last_updated.as_deref().unwrap_or("never")),
                ]),
                Line::from(vec![Span::raw("")]),
                Line::from(vec![Span::raw(" u  Update/Download  e  Toggle  x  Remove")]),
                Line::from(vec![Span::raw(" a  Add new subscription")]),
            ]
        } else {
            vec![
                Line::from(vec![Span::raw("No subscriptions configured.")]),
                Line::from(vec![Span::raw("")]),
                Line::from(vec![Span::raw("Press 'a' to add one.")]),
                Line::from(vec![Span::raw("Or edit:")]),
                Line::from(vec![Span::raw(AppState::subscriptions_path())]),
            ]
        };

        let para = Paragraph::new(detail_lines)
            .block(Block::default().borders(Borders::ALL).title(" Detail "))
            .style(Style::default().fg(theme.fg));

        frame.render_widget(para, right);
    }

    fn handle_key(&mut self, key: KeyEvent, state: &AppState) -> Vec<Action> {
        match self.input_mode {
            InputMode::AddUrl => match key.code {
                KeyCode::Esc => {
                    self.input_mode = InputMode::None;
                    self.input_text.clear();
                    self.cursor_pos = 0;
                    self.scroll_offset = 0;
                    vec![]
                }
                KeyCode::Enter => {
                    let url = self.input_text.trim().to_string();
                    self.input_mode = InputMode::None;
                    self.input_text.clear();
                    self.cursor_pos = 0;
                    self.scroll_offset = 0;
                    if !url.is_empty() {
                        let name = url
                            .split('/')
                            .last()
                            .unwrap_or("subscription")
                            .split('?')
                            .next()
                            .unwrap_or("sub")
                            .to_string();
                        vec![Action::AddSubscription { name, url }]
                    } else {
                        vec![]
                    }
                }
                KeyCode::Left => {
                    self.cursor_pos = self.cursor_pos.saturating_sub(1);
                    vec![]
                }
                KeyCode::Right => {
                    self.cursor_pos = (self.cursor_pos + 1).min(self.input_text.len());
                    vec![]
                }
                KeyCode::Home => {
                    self.cursor_pos = 0;
                    vec![]
                }
                KeyCode::End => {
                    self.cursor_pos = self.input_text.len();
                    vec![]
                }
                KeyCode::Backspace => {
                    if self.cursor_pos > 0 {
                        self.cursor_pos -= 1;
                        self.input_text.remove(self.cursor_pos);
                    }
                    vec![]
                }
                KeyCode::Delete => {
                    if self.cursor_pos < self.input_text.len() {
                        self.input_text.remove(self.cursor_pos);
                    }
                    vec![]
                }
                KeyCode::Char(c)
                    if key.modifiers == KeyModifiers::NONE
                        || key.modifiers == KeyModifiers::SHIFT =>
                {
                    self.input_text.insert(self.cursor_pos, c);
                    self.cursor_pos += 1;
                    vec![]
                }
                _ => vec![],
            },
            InputMode::None => match key.code {
                KeyCode::Down | KeyCode::Char('j') => {
                    let max = state.subscriptions.len().saturating_sub(1);
                    self.selected = (self.selected + 1).min(max);
                    vec![]
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.selected = self.selected.saturating_sub(1);
                    vec![]
                }
                KeyCode::Char('u') => {
                    if let Some(sub) = state.subscriptions.get(self.selected) {
                        vec![Action::DownloadSubscription(sub.url.clone())]
                    } else {
                        vec![]
                    }
                }
                KeyCode::Char('e') => {
                    if let Some(sub) = state.subscriptions.get(self.selected) {
                        vec![Action::ToggleSubscription(sub.name.clone())]
                    } else {
                        vec![]
                    }
                }
                KeyCode::Char('a') => {
                    self.input_mode = InputMode::AddUrl;
                    vec![]
                }
                KeyCode::Char('x') | KeyCode::Delete => {
                    if let Some(sub) = state.subscriptions.get(self.selected) {
                        vec![Action::RemoveSubscription(sub.name.clone())]
                    } else {
                        vec![]
                    }
                }
                _ => vec![],
            },
        }
    }

    fn is_modal(&self) -> bool {
        !matches!(self.input_mode, InputMode::None)
    }

    fn page_id(&self) -> PageId {
        PageId::Subscriptions
    }

    fn title(&self) -> &'static str {
        "Subscriptions"
    }
}
