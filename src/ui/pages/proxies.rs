use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use std::collections::HashMap;

use super::super::layout;
use super::super::theme::Theme;
use super::{Page, PageId};
use crate::clash::types::ProxyGroup;
use crate::event::action::Action;
use crate::state::store::AppState;

/// Format proxy type for display: SS, VMess, Trojan, etc.
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
            group_state: ListState::default(),
            proxy_state: ListState::default(),
        }
    }

    fn group_count(&self, state: &AppState) -> usize {
        state.proxy_groups.len()
    }

    fn selected_group<'a>(&self, state: &'a AppState) -> Option<(&'a String, &'a ProxyGroup)> {
        selected_group_at(state, self.group_index)
    }

    fn proxy_count(&self, state: &AppState) -> usize {
        self.selected_group(state)
            .map(|(_, g)| g.all.len())
            .unwrap_or(0)
    }
}

impl Page for ProxiesPage {
    fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme, state: &AppState) {
        let (left, right) = layout::two_panels(area);

        // Build sorted group list
        let mut groups: Vec<(&String, &ProxyGroup)> = state.proxy_groups.iter().collect();
        groups.sort_by(|a, b| a.0.cmp(b.0));

        // Left panel: proxy groups
        let group_items: Vec<ListItem> = groups
            .iter()
            .enumerate()
            .map(|(i, (name, group))| {
                let type_label = match group.group_type.as_str() {
                    "URLTest" => " [url-test]",
                    "Fallback" => " [fallback]",
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
                ListItem::new(Line::from(vec![
                    Span::styled(format!("{}", name), style),
                    Span::styled(type_label, Style::default().fg(theme.latency_offline)),
                ]))
            })
            .collect();

        let mut group_state = self.group_state.clone();
        let group_list = List::new(group_items)
            .block(Block::default().borders(Borders::ALL).title(" Groups "))
            .highlight_style(
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::REVERSED),
            )
            .highlight_symbol("> ");

        frame.render_stateful_widget(group_list, left, &mut group_state);

        // Right panel: proxies in selected group
        if let Some((name, group)) = self.selected_group(state) {
            // Build sorted index list
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
                    let prefix = if is_active { "✓ " } else { "  " };

                    let style = if pos == self.proxy_index && self.focus == PanelFocus::Proxies {
                        Style::default().fg(theme.accent)
                    } else if is_active {
                        Style::default()
                            .fg(theme.success)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.fg)
                    };

                    let ptype = proxy_type_for(proxy_name, &state.proxy_groups)
                        .map(|t| format!("[{}]", proxy_type_label(&t)))
                        .unwrap_or_default();
                    let lat = latency_span(proxy_name, &state.proxy_groups, theme);

                    ListItem::new(Line::from(vec![
                        Span::styled(format!("{}{}", prefix, proxy_name), style),
                        Span::styled(
                            format!(" {}", ptype),
                            Style::default().fg(theme.latency_offline),
                        ),
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
            let empty = Paragraph::new("No proxy groups available")
                .block(Block::default().borders(Borders::ALL).title(" Proxies "))
                .style(Style::default().fg(theme.latency_offline));
            frame.render_widget(empty, right);
        }
    }

    fn handle_key(&mut self, key: KeyEvent, state: &AppState) -> Vec<Action> {
        match self.focus {
            PanelFocus::Groups => match key.code {
                KeyCode::Down | KeyCode::Char('j') => {
                    self.group_index =
                        (self.group_index + 1).min(self.group_count(state).saturating_sub(1));
                    self.proxy_index = 0;
                    vec![]
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.group_index = self.group_index.saturating_sub(1);
                    self.proxy_index = 0;
                    vec![]
                }
                KeyCode::Enter => {
                    self.focus = PanelFocus::Proxies;
                    vec![]
                }
                KeyCode::Tab => {
                    self.focus = PanelFocus::Proxies;
                    vec![]
                }
                KeyCode::Char('t') => {
                    if let Some((name, group)) = selected_group_at(state, self.group_index) {
                        self.sort_by_latency = true;
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
                KeyCode::Esc => {
                    self.focus = PanelFocus::Groups;
                    vec![]
                }
                KeyCode::Enter => {
                    if let Some((group_name, group)) = selected_group_at(state, self.group_index) {
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
                    if let Some((group_name, group)) = selected_group_at(state, self.group_index) {
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

fn selected_group_at(state: &AppState, index: usize) -> Option<(&String, &ProxyGroup)> {
    let mut groups: Vec<(&String, &ProxyGroup)> = state.proxy_groups.iter().collect();
    groups.sort_by(|a, b| a.0.cmp(b.0));
    groups.get(index).copied()
}

/// Get color-coded latency span for a proxy node.
fn latency_span(
    proxy_name: &str,
    all_groups: &HashMap<String, ProxyGroup>,
    theme: &Theme,
) -> Span<'static> {
    if let Some(proxy_group) = all_groups.get(proxy_name) {
        if let Some(ref history) = proxy_group.history {
            if let Some(last) = history.last() {
                let delay = last.delay;
                let color = if delay < 100 {
                    theme.latency_low
                } else if delay < 300 {
                    theme.latency_mid
                } else {
                    theme.latency_high
                };
                return Span::styled(format!(" {}ms", delay), Style::default().fg(color));
            }
        }
    }
    Span::styled(" -", Style::default().fg(theme.latency_offline))
}

/// Get the proxy type for a proxy node by looking it up in the full proxies map.
fn proxy_type_for(name: &str, all_groups: &HashMap<String, ProxyGroup>) -> Option<String> {
    all_groups.get(name).map(|p| p.group_type.clone())
}
