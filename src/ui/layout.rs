use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::Block;

/// Split area into tab bar (1 line), content, and status bar (1 line)
pub fn app_layout(area: Rect) -> (Rect, Rect, Rect) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);
    (layout[0], layout[1], layout[2])
}

/// Split content area into left (30%) and right (70%) panels
pub fn two_panels(area: Rect) -> (Rect, Rect) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);
    (layout[0], layout[1])
}

/// Create a card-style bordered block
pub fn card<'a>(title: &'a str) -> Block<'a> {
    Block::bordered().title(title)
}
