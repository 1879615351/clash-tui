use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use super::super::theme::Theme;
use crate::ui::pages::PageId;

pub fn draw(frame: &mut Frame, area: Rect, theme: &Theme, active_page: PageId) {
    let help_text = match active_page {
        PageId::Dashboard => vec![
            Line::from(vec![
                Span::styled(
                    " Tab ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Switch page  "),
                Span::styled(
                    " q ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Quit  "),
                Span::styled(
                    " ? ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Help  "),
            ]),
            Line::from(vec![
                Span::styled(
                    " m ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Cycle mode  "),
                Span::styled(
                    " r ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Refresh  "),
            ]),
        ],
        PageId::Proxies => vec![
            Line::from(vec![
                Span::styled(
                    " j/k/↑↓ ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Navigate  "),
                Span::styled(
                    " t ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Test latency  "),
            ]),
            Line::from(vec![
                Span::styled(
                    " Enter/→ ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Select/into  "),
                Span::styled(
                    " Esc/← ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Back  "),
            ]),
            Line::from(vec![
                Span::styled(
                    " Tab ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Switch page  "),
                Span::styled(
                    " q ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Quit  "),
            ]),
        ],
        PageId::Connections => vec![
            Line::from(vec![
                Span::styled(
                    " j/k/↑↓ ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Navigate  "),
                Span::styled(
                    " d ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Close selected  "),
                Span::styled(
                    " D ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Close all  "),
            ]),
            Line::from(vec![
                Span::styled(
                    " Tab ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Switch page  "),
                Span::styled(
                    " q ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Quit  "),
            ]),
        ],
        PageId::Logs => vec![
            Line::from(vec![
                Span::styled(
                    " j/k/↑↓ ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Scroll  "),
                Span::styled(
                    " e/w/i ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Filter: error/warn/info  "),
            ]),
            Line::from(vec![
                Span::styled(
                    " a ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Show all  "),
                Span::styled(
                    " Home/End ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Jump top/bottom  "),
                Span::styled(
                    " q ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Quit  "),
            ]),
            Line::from(vec![
                Span::styled(
                    " Tab ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Switch page  "),
            ]),
        ],
        PageId::Rules => vec![Line::from(vec![
            Span::styled(
                " j/k/↑↓ ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Navigate  "),
            Span::styled(
                " Tab ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Switch page  "),
            Span::styled(
                " q ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Quit  "),
        ])],
        PageId::Settings => vec![
            Line::from(vec![
                Span::styled(
                    " p ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Toggle proxy  "),
                Span::styled(
                    " Tab ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Switch page  "),
            ]),
            Line::from(vec![
                Span::styled(
                    " ? ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Close help  "),
                Span::styled(
                    " q ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Quit  "),
            ]),
        ],
        PageId::Subscriptions => vec![
            Line::from(vec![
                Span::styled(
                    " j/k/↑↓ ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Navigate  "),
                Span::styled(
                    " u ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Update  "),
                Span::styled(
                    " e ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Toggle  "),
            ]),
            Line::from(vec![
                Span::styled(
                    " Tab ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Switch page  "),
                Span::styled(
                    " q ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Quit  "),
            ]),
        ],
    };

    let text = Text::from(help_text);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Help (press ? to close) ")
        .style(Style::default().bg(theme.surface));

    let paragraph = Paragraph::new(text).block(block).alignment(Alignment::Left);

    // Center the help box
    let help_area = centered_rect(area, 60, 40);
    frame.render_widget(Clear, help_area);
    frame.render_widget(paragraph, help_area);
}

fn centered_rect(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
