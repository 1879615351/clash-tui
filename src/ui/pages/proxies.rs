use std::cell::Cell;
use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use super::super::theme::Theme;
use super::{Page, PageId};
use crate::clash::types::ProxyGroup;
use crate::event::action::Action;
use crate::state::store::AppState;

fn proxy_type_label(ptype: &str) -> &str {
    match ptype {
        "ss" => "SS",
        "vmess" => "VMess",
        "vless" => "VLESS",
        "trojan" => "Trojan",
        "hysteria" | "hysteria2" => "Hysteria",
        "tuic" => "TUIC",
        "shadowsocks" => "SS",
        "http" => "HTTP",
        "socks5" => "SOCKS5",
        "snell" => "Snell",
        "wireguard" => "WG",
        _ => ptype,
    }
}

fn is_proxy_group(g: &ProxyGroup) -> bool {
    matches!(
        g.group_type.as_str(),
        "Selector" | "URLTest" | "Fallback" | "LoadBalance"
    )
}

#[derive(PartialEq)]
enum PanelFocus {
    Groups,
    Proxies,
}

pub struct ProxiesPage {
    group_index: usize,
    proxy_index: usize,
    focus: PanelFocus,
    sort_by_latency: bool,
    testing: Cell<bool>,
    group_scroll: Cell<usize>,
    proxy_scroll: Cell<usize>,
    group_state: ListState,
    proxy_state: ListState,
}

impl ProxiesPage {
    pub fn new() -> Self {
        Self {
            group_index: 0,
            proxy_index: 0,
            focus: PanelFocus::Groups,
            sort_by_latency: false,
            testing: Cell::new(false),
            group_scroll: Cell::new(0),
            proxy_scroll: Cell::new(0),
            group_state: ListState::default(),
            proxy_state: ListState::default(),
        }
    }

    fn group_count(&self, state: &AppState) -> usize {
        state
            .proxy_groups
            .values()
            .filter(|g| is_proxy_group(g))
            .count()
    }

    fn selected_group<'a>(&self, state: &'a AppState) -> Option<(&'a String, &'a ProxyGroup)> {
        groups_at(state).get(self.group_index).copied()
    }

    fn proxy_count(&self, state: &AppState) -> usize {
        self.selected_group(state)
            .map(|(_, g)| g.all.len())
            .unwrap_or(0)
    }
}

impl Page for ProxiesPage {
    fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme, state: &AppState) {
        // Reserve bottom line for key hints
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(area);
        let content_area = chunks[0];
        let footer_area = chunks[1];

        let panels = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(content_area);
        let left = panels[0];
        let right = panels[1];

        // === LEFT: proxy groups ===
        let groups = groups_at(state);
        let group_items: Vec<ListItem> = groups
            .iter()
            .enumerate()
            .map(|(i, (name, group))| {
                let type_label = match group.group_type.as_str() {
                    "URLTest" => " [url]",
                    "Fallback" => " [fb]",
                    "LoadBalance" => " [lb]",
                    _ => "",
                };
                let style = if i == self.group_index && self.focus == PanelFocus::Groups {
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.fg)
                };
                let current = group.now.as_deref().unwrap_or("?");
                ListItem::new(Line::from(vec![
                    Span::styled(name.to_string(), style),
                    Span::styled(
                        format!(" → {} {}", current, type_label),
                        Style::default().fg(theme.latency_offline),
                    ),
                ]))
            })
            .collect();

        let mut group_state = self.group_state.clone();
        let g_visible = left.height.saturating_sub(2) as usize;
        let g_scroll = scroll_offset(self.group_index, g_visible);
        self.group_scroll.set(g_scroll);
        *group_state.offset_mut() = g_scroll;

        let group_list = List::new(group_items)
            .block(Block::default().borders(Borders::ALL).title(" Groups "))
            .highlight_style(
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::REVERSED),
            )
            .highlight_symbol("> ");

        frame.render_stateful_widget(group_list, left, &mut group_state);

        // === RIGHT: proxies in selected group ===
        if let Some((name, group)) = self.selected_group(state) {
            let mut indices: Vec<usize> = (0..group.all.len()).collect();
            if self.sort_by_latency {
                indices.sort_by_key(|&i| {
                    let name = &group.all[i];
                    state
                        .proxy_groups
                        .get(name)
                        .and_then(|p| p.history.as_ref())
                        .and_then(|h| h.last())
                        .map(|r| r.delay)
                        .unwrap_or(u64::MAX)
                });
            }

            let proxy_items: Vec<ListItem> = indices
                .iter()
                .enumerate()
                .map(|(pos, &i)| {
                    let proxy_name = &group.all[i];
                    let is_active = group.now.as_deref() == Some(proxy_name.as_str());
                    let prefix = if is_active { " ◆ " } else { "   " };

                    let style = if pos == self.proxy_index && self.focus == PanelFocus::Proxies {
                        if is_active {
                            Style::default()
                                .fg(theme.surface)
                                .bg(theme.success)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(theme.accent)
                        }
                    } else if is_active {
                        Style::default()
                            .fg(theme.success)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.fg)
                    };

                    let ptype = proxy_info(proxy_name, &state.proxy_groups);
                    let lat = latency_span(proxy_name, state, theme);

                    ListItem::new(Line::from(vec![
                        Span::styled(format!("{}{}", prefix, proxy_name), style),
                        Span::styled(ptype, Style::default().fg(theme.latency_offline)),
                        lat,
                    ]))
                })
                .collect();

            let sort_label = if self.sort_by_latency {
                " [sorted]"
            } else {
                ""
            };
            let title = format!(" {} ({} proxies){} ", name, group.all.len(), sort_label);

            let mut proxy_state = self.proxy_state.clone();
            let p_visible = right.height.saturating_sub(2) as usize;
            let p_scroll = scroll_offset(self.proxy_index, p_visible);
            self.proxy_scroll.set(p_scroll);
            *proxy_state.offset_mut() = p_scroll;

            let proxy_list = List::new(proxy_items)
                .block(Block::default().borders(Borders::ALL).title(title))
                .highlight_style(
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::REVERSED),
                )
                .highlight_symbol("> ");

            frame.render_stateful_widget(proxy_list, right, &mut proxy_state);
        } else {
            let empty = Paragraph::new("No proxy groups")
                .block(Block::default().borders(Borders::ALL).title(" Proxies "))
                .style(Style::default().fg(theme.latency_offline));
            frame.render_widget(empty, right);
        }

        // === FOOTER: key hints ===
        let keys = if self.focus == PanelFocus::Groups {
            vec![
                Span::styled(
                    " j/k ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("navigate  "),
                Span::styled(
                    " Enter/→ ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("into proxies  "),
                Span::styled(
                    " t ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("test all  "),
                Span::styled(
                    " s ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("sort  "),
                Span::styled(
                    " Tab ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("next page"),
            ]
        } else {
            vec![
                Span::styled(
                    " j/k ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("navigate  "),
                Span::styled(
                    " Enter ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("switch  "),
                Span::styled(
                    " t ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("test  "),
                Span::styled(
                    " Esc/← ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("back  "),
                Span::styled(
                    " Tab ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("next page"),
            ]
        };

        let footer = Paragraph::new(Line::from(keys)).style(Style::default().fg(theme.fg));
        frame.render_widget(footer, footer_area);
    }

    fn handle_key(&mut self, key: KeyEvent, state: &AppState) -> Vec<Action> {
        match self.focus {
            PanelFocus::Groups => match key.code {
                KeyCode::Down | KeyCode::Char('j') => {
                    let max = self.group_count(state).saturating_sub(1);
                    self.group_index = (self.group_index + 1).min(max);
                    self.proxy_index = 0;
                    vec![]
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.group_index = self.group_index.saturating_sub(1);
                    self.proxy_index = 0;
                    vec![]
                }
                KeyCode::Enter | KeyCode::Right => {
                    self.focus = PanelFocus::Proxies;
                    vec![]
                }
                KeyCode::Char('t') => {
                    self.sort_by_latency = true;
                    self.testing.set(true);
                    if let Some((name, group)) = groups_at(state).get(self.group_index).copied() {
                        vec![Action::TestAllLatency {
                            group: name.clone(),
                            proxies: group.all.clone(),
                        }]
                    } else {
                        vec![]
                    }
                }
                KeyCode::Char('s') => {
                    self.sort_by_latency = !self.sort_by_latency;
                    vec![]
                }
                _ => vec![],
            },
            PanelFocus::Proxies => match key.code {
                KeyCode::Down | KeyCode::Char('j') => {
                    let max = self.proxy_count(state).saturating_sub(1);
                    self.proxy_index = (self.proxy_index + 1).min(max);
                    vec![]
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.proxy_index = self.proxy_index.saturating_sub(1);
                    vec![]
                }
                KeyCode::Esc | KeyCode::Left => {
                    self.focus = PanelFocus::Groups;
                    vec![]
                }
                KeyCode::Enter => {
                    if let Some((group_name, group)) =
                        groups_at(state).get(self.group_index).copied()
                    {
                        if let Some(proxy_name) = group.all.get(self.proxy_index) {
                            return vec![Action::SelectProxy {
                                group: group_name.clone(),
                                proxy: proxy_name.clone(),
                            }];
                        }
                    }
                    vec![]
                }
                KeyCode::Char('t') => {
                    if let Some((group_name, group)) =
                        groups_at(state).get(self.group_index).copied()
                    {
                        if let Some(proxy_name) = group.all.get(self.proxy_index) {
                            return vec![Action::TestLatency {
                                group: group_name.clone(),
                                proxy: proxy_name.clone(),
                            }];
                        }
                    }
                    vec![]
                }
                _ => vec![],
            },
        }
    }

    fn page_id(&self) -> PageId {
        PageId::Proxies
    }

    fn title(&self) -> &'static str {
        "Proxies"
    }
}

fn groups_at(state: &AppState) -> Vec<(&String, &ProxyGroup)> {
    let mut groups: Vec<_> = state
        .proxy_groups
        .iter()
        .filter(|(_, g)| is_proxy_group(g))
        .collect();
    groups.sort_by(|a, b| a.0.cmp(b.0));
    groups
}

/// Calculate scroll offset so the selected item is always visible.
/// Cursor moves within the visible area first, then content scrolls.
fn scroll_offset(selected: usize, visible: usize) -> usize {
    if visible == 0 {
        return 0;
    }
    if selected < visible {
        0
    } else {
        selected.saturating_sub(visible).saturating_add(1)
    }
}

fn latency_span(proxy_name: &str, state: &AppState, theme: &Theme) -> Span<'static> {
    // 1. Check our frontend cache first (immediate results from t/TestLatency)
    if let Some(&delay) = state.latency_cache.get(proxy_name) {
        let color = latency_color(delay, theme);
        return Span::styled(format!(" {}ms", delay), Style::default().fg(color));
    }
    // 2. Fall back to mihomo's history
    if let Some(proxy_group) = state.proxy_groups.get(proxy_name) {
        if let Some(ref history) = proxy_group.history {
            if let Some(last) = history.last() {
                let color = latency_color(last.delay, theme);
                return Span::styled(format!(" {}ms", last.delay), Style::default().fg(color));
            }
        }
    }
    Span::styled(" -", Style::default().fg(theme.latency_offline))
}

fn latency_color(delay: u64, theme: &Theme) -> ratatui::style::Color {
    if delay < 100 {
        theme.latency_low
    } else if delay < 300 {
        theme.latency_mid
    } else {
        theme.latency_high
    }
}

fn proxy_info(name: &str, all_groups: &HashMap<String, ProxyGroup>) -> String {
    if let Some(p) = all_groups.get(name) {
        let label = proxy_type_label(&p.group_type);
        format!(" [{}]", label)
    } else {
        String::new()
    }
}
