use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Tabs};
use ratatui::Frame;

use super::super::pages::PageId;
use super::super::theme::Theme;

pub fn draw(frame: &mut Frame, area: Rect, pages: &[PageId], active: PageId, theme: &Theme) {
    let titles: Vec<Line> = pages
        .iter()
        .map(|p| {
            if *p == active {
                Line::from(format!("  {}  ", p.label())).style(
                    Style::default()
                        .fg(theme.tab_active)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Line::from(format!("  {}  ", p.label()))
                    .style(Style::default().fg(theme.tab_inactive))
            }
        })
        .collect();

    let position = pages.iter().position(|p| *p == active).unwrap_or(0);
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::BOTTOM))
        .highlight_style(Style::default().fg(theme.accent))
        .select(position);

    frame.render_widget(tabs, area);
}
