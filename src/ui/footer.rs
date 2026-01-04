use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line as RatatuiLine, Span},
};

use crate::app::{App, InputMode, ViewMode};
use crate::registry::{FooterMode, KeyAction, footer_actions};

pub fn render_footer(app: &App) -> RatatuiLine<'static> {
    match (&app.view, &app.input_mode) {
        (_, InputMode::Command) => RatatuiLine::from(vec![
            Span::styled(":", Style::default().fg(Color::Blue)),
            Span::raw(app.command_buffer.content().to_string()),
        ]),
        (_, InputMode::QueryInput) => {
            let buffer = match &app.view {
                ViewMode::Filter(state) => state.query_buffer.content(),
                ViewMode::Daily(_) => app.command_buffer.content(),
            };
            RatatuiLine::from(vec![
                Span::styled("/", Style::default().fg(Color::Magenta)),
                Span::raw(buffer.to_string()),
            ])
        }
        (_, InputMode::Edit(_)) => build_footer_line(" EDIT ", Color::Green, FooterMode::Edit),
        (_, InputMode::Reorder) => {
            build_footer_line(" REORDER ", Color::Yellow, FooterMode::Reorder)
        }
        (_, InputMode::Confirm(_)) => RatatuiLine::from(vec![
            Span::styled(
                " CONFIRM ",
                Style::default().fg(Color::Black).bg(Color::Blue),
            ),
            Span::styled("  y", Style::default().fg(Color::Gray)),
            Span::styled(" Yes  ", Style::default().fg(Color::DarkGray)),
            Span::styled("n/Esc", Style::default().fg(Color::Gray)),
            Span::styled(" No", Style::default().fg(Color::DarkGray)),
        ]),
        (_, InputMode::Selection(state)) => {
            let count = state.count();
            build_footer_line(
                &format!(" SELECT ({count}) "),
                Color::Green,
                FooterMode::Selection,
            )
        }
        (ViewMode::Daily(_), InputMode::Normal) => {
            build_footer_line(" DAILY ", Color::Cyan, FooterMode::NormalDaily)
        }
        (ViewMode::Filter(_), InputMode::Normal) => {
            build_footer_line(" FILTER ", Color::Magenta, FooterMode::NormalFilter)
        }
    }
}

fn build_footer_line(mode_name: &str, color: Color, mode: FooterMode) -> RatatuiLine<'static> {
    let mut spans = vec![Span::styled(
        mode_name.to_string(),
        Style::default().fg(Color::Black).bg(color),
    )];

    for action in footer_actions(mode) {
        spans.extend(action_spans(action));
    }

    RatatuiLine::from(spans)
}

fn action_spans(action: &KeyAction) -> [Span<'static>; 2] {
    let key_display = match action.alt_key {
        Some(alt) => format!("{}/{}", action.key, alt),
        None => action.key.to_string(),
    };
    [
        Span::styled(format!("  {key_display}"), Style::default().fg(Color::Gray)),
        Span::styled(
            format!(" {} ", action.short_text),
            Style::default().fg(Color::DarkGray),
        ),
    ]
}

pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
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
        .split(vertical[1])[1]
}
