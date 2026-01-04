use std::sync::LazyLock;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line as RatatuiLine, Span},
};

use crate::app::{App, InputMode, ViewMode};

pub fn render_footer(app: &App) -> RatatuiLine<'static> {
    match (&app.view, &app.input_mode) {
        (_, InputMode::Command) => RatatuiLine::from(vec![
            Span::styled(":", Style::default().fg(Color::Yellow)),
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
        (_, InputMode::Edit(_)) => RatatuiLine::from(vec![
            Span::styled(" EDIT ", Style::default().fg(Color::Black).bg(Color::Green)),
            Span::styled("  Enter", Style::default().fg(Color::Gray)),
            Span::styled(" Save  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Tab", Style::default().fg(Color::Gray)),
            Span::styled(" Save and new  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Shift+Tab", Style::default().fg(Color::Gray)),
            Span::styled(" Toggle entry type  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Esc", Style::default().fg(Color::Gray)),
            Span::styled(" Cancel", Style::default().fg(Color::DarkGray)),
        ]),
        (_, InputMode::Reorder) => RatatuiLine::from(vec![
            Span::styled(
                " REORDER ",
                Style::default().fg(Color::Black).bg(Color::Yellow),
            ),
            Span::styled("  j/k|↕", Style::default().fg(Color::Gray)),
            Span::styled(" Move down/up  ", Style::default().fg(Color::DarkGray)),
            Span::styled("r/Enter", Style::default().fg(Color::Gray)),
            Span::styled(" Save  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Esc", Style::default().fg(Color::Gray)),
            Span::styled(" Cancel", Style::default().fg(Color::DarkGray)),
        ]),
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
        (ViewMode::Daily(_), InputMode::Normal) => RatatuiLine::from(vec![
            Span::styled(" DAILY ", Style::default().fg(Color::Black).bg(Color::Cyan)),
            Span::styled("  Enter", Style::default().fg(Color::Gray)),
            Span::styled(" New entry  ", Style::default().fg(Color::DarkGray)),
            Span::styled("i", Style::default().fg(Color::Gray)),
            Span::styled(" Edit entry  ", Style::default().fg(Color::DarkGray)),
            Span::styled("c", Style::default().fg(Color::Gray)),
            Span::styled(" Toggle task  ", Style::default().fg(Color::DarkGray)),
            Span::styled("/", Style::default().fg(Color::Gray)),
            Span::styled(" Filter  ", Style::default().fg(Color::DarkGray)),
            Span::styled("?", Style::default().fg(Color::Gray)),
            Span::styled(" Help", Style::default().fg(Color::DarkGray)),
        ]),
        (ViewMode::Filter(_), InputMode::Normal) => RatatuiLine::from(vec![
            Span::styled(
                " FILTER ",
                Style::default().fg(Color::Black).bg(Color::Magenta),
            ),
            Span::styled("  c", Style::default().fg(Color::Gray)),
            Span::styled(" Toggle  ", Style::default().fg(Color::DarkGray)),
            Span::styled("x", Style::default().fg(Color::Gray)),
            Span::styled(" Delete  ", Style::default().fg(Color::DarkGray)),
            Span::styled("r", Style::default().fg(Color::Gray)),
            Span::styled(" Refresh  ", Style::default().fg(Color::DarkGray)),
            Span::styled("v", Style::default().fg(Color::Gray)),
            Span::styled(" View day  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Esc", Style::default().fg(Color::Gray)),
            Span::styled(" Exit  ", Style::default().fg(Color::DarkGray)),
            Span::styled("?", Style::default().fg(Color::Gray)),
            Span::styled(" Help", Style::default().fg(Color::DarkGray)),
        ]),
    }
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

const HELP_KEY_WIDTH: usize = 14;
const HELP_GUTTER_WIDTH: usize = 2;

static HELP_LINES: LazyLock<Vec<RatatuiLine<'static>>> = LazyLock::new(build_help_lines);

fn build_help_lines() -> Vec<RatatuiLine<'static>> {
    let header_style = Style::default().fg(Color::Cyan);
    let key_style = Style::default().fg(Color::Yellow);
    let desc_style = Style::default().fg(Color::White);
    let header_indent = " ".repeat(HELP_KEY_WIDTH + HELP_GUTTER_WIDTH);

    let sections: &[(&str, &[(&str, &str)])] = &[
        (
            "[Daily]",
            &[
                ("Enter", "New entry at end"),
                ("o/O", "New entry below/above"),
                ("i", "Edit selected"),
                ("c", "Toggle task complete"),
                ("x", "Delete entry"),
                ("y", "Yank to clipboard"),
                ("u", "Undo delete"),
                ("j/k", "Navigate down/up"),
                ("g/G", "Jump to first/last"),
                ("h/l|[]", "Previous/next day"),
                ("t", "Go to today"),
                ("s", "Sort entries"),
                ("r", "Reorder mode"),
                ("z", "Toggle hide completed"),
                ("/", "Filter mode"),
                ("Tab", "Return to filter"),
                ("0-9", "Filter favorite tag"),
                ("`", "Toggle Global/Project journal"),
                (":", "Command mode"),
            ],
        ),
        (
            "[Reorder]",
            &[
                ("j/k|↕", "Move entry down/up"),
                ("r/Enter", "Save"),
                ("Esc", "Cancel"),
            ],
        ),
        (
            "[Edit]",
            &[
                ("Enter", "Save and exit"),
                ("Tab", "Save and new"),
                ("Shift+Tab", "Toggle entry type"),
                ("Esc", "Cancel"),
            ],
        ),
        (
            "[Text Editing]",
            &[
                ("←/→", "Move cursor left/right"),
                ("Alt+B/F", "Move cursor one word left/right"),
                ("Home/Ctrl+A", "Move cursor to start"),
                ("End/Ctrl+E", "Move cursor to end"),
                ("Ctrl+W", "Delete word before cursor"),
                ("Alt+D", "Delete from cursor to end of word"),
                ("Ctrl+U", "Delete from cursor to start"),
                ("Ctrl+K", "Delete from cursor to end"),
                ("Delete", "Delete char after cursor"),
            ],
        ),
        (
            "[Filter]",
            &[
                ("j/k|↕", "Navigate down/up"),
                ("g/G", "Jump first/last"),
                ("Enter", "Quick add to today"),
                ("i", "Edit entry"),
                ("c", "Toggle task"),
                ("x", "Delete entry"),
                ("y", "Yank to clipboard"),
                ("r", "Refresh results"),
                ("v", "View day"),
                ("/", "Edit filter"),
                ("Tab/Esc", "Exit to daily"),
            ],
        ),
        (
            "[Filter Syntax]",
            &[
                ("!tasks", "Incomplete tasks"),
                ("!tasks/done", "Completed tasks"),
                ("!notes", "Notes only"),
                ("!events", "Events only"),
                ("#tag", "Filter by tag"),
                ("$name", "Saved filter"),
                ("@before:DATE", "Before date"),
                ("@after:DATE", "After date"),
                ("@overdue", "Has past @date"),
                (
                    "DATE:",
                    "MM/DD, tomorrow, yesterday, next-mon, last-fri, 3d, -3d",
                ),
            ],
        ),
        (
            "[Commands]",
            &[
                (":[d]ate", "Go to date (MM/DD)"),
                (":[g]lobal", "Switch to Global journal"),
                (":[p]roject", "Manage project journal [init|default|path]"),
                (":[c]onfig", "Manage config [reload]"),
                (":[q]uit", "Quit"),
            ],
        ),
    ];

    let mut lines = Vec::new();

    for (i, (title, keys)) in sections.iter().enumerate() {
        lines.push(RatatuiLine::from(Span::styled(
            format!("{header_indent}{title}"),
            header_style,
        )));
        for (key, desc) in *keys {
            lines.push(help_line(key, desc, key_style, desc_style));
        }
        if i < sections.len() - 1 {
            lines.push(RatatuiLine::from(""));
        }
    }

    lines
}

fn help_line(key: &str, desc: &str, key_style: Style, desc_style: Style) -> RatatuiLine<'static> {
    RatatuiLine::from(vec![
        Span::styled(
            format!(
                "{:>width$}{}",
                key,
                " ".repeat(HELP_GUTTER_WIDTH),
                width = HELP_KEY_WIDTH
            ),
            key_style,
        ),
        Span::styled(desc.to_string(), desc_style),
    ])
}

#[must_use]
pub fn get_help_total_lines() -> usize {
    HELP_LINES.len()
}

pub fn render_help_content(scroll: usize, visible_height: usize) -> Vec<RatatuiLine<'static>> {
    HELP_LINES
        .iter()
        .skip(scroll)
        .take(visible_height)
        .cloned()
        .collect()
}
