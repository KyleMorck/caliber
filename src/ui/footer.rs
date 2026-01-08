use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line as RatatuiLine, Span},
};

use crate::app::{App, InputMode, ViewMode};
use crate::dispatch::Keymap;
use crate::registry::{FooterMode, KeyAction, KeyContext, footer_actions};

pub fn render_footer(app: &App) -> RatatuiLine<'static> {
    match (&app.view, &app.input_mode) {
        (_, InputMode::Command) => RatatuiLine::from(vec![
            Span::styled(":", Style::default().fg(Color::Cyan)),
            Span::styled(
                app.command_buffer.content().to_string(),
                Style::default().fg(Color::Cyan),
            ),
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
        (_, InputMode::Edit(_)) => {
            build_footer_line(" EDIT ", Color::Green, FooterMode::Edit, &app.keymap)
        }
        (_, InputMode::Reorder) => {
            build_footer_line(" REORDER ", Color::Yellow, FooterMode::Reorder, &app.keymap)
        }
        (_, InputMode::Confirm(_)) => RatatuiLine::from(vec![
            Span::styled(
                " CONFIRM ",
                Style::default().fg(Color::Black).bg(Color::Blue),
            ),
            Span::styled("  y", Style::default().fg(Color::Gray)),
            Span::styled(" Yes  ", Style::default().dim()),
            Span::styled("n/Esc", Style::default().fg(Color::Gray)),
            Span::styled(" No", Style::default().dim()),
        ]),
        (_, InputMode::Selection(state)) => {
            let count = state.count();
            build_footer_line(
                &format!(" SELECT ({count}) "),
                Color::Green,
                FooterMode::Selection,
                &app.keymap,
            )
        }
        (_, InputMode::Datepicker(_)) => {
            build_footer_line(" DATE ", Color::Cyan, FooterMode::Datepicker, &app.keymap)
        }
        (_, InputMode::ProjectPicker(_)) => RatatuiLine::from(vec![
            Span::styled(
                " PROJECTS ",
                Style::default().fg(Color::Black).bg(Color::Cyan),
            ),
            Span::styled("  Enter", Style::default().fg(Color::Gray)),
            Span::styled(" Select  ", Style::default().dim()),
            Span::styled("Esc", Style::default().fg(Color::Gray)),
            Span::styled(" Close", Style::default().dim()),
        ]),
        (ViewMode::Daily(_), InputMode::Normal) => {
            build_footer_line(" DAILY ", Color::Cyan, FooterMode::NormalDaily, &app.keymap)
        }
        (ViewMode::Filter(_), InputMode::Normal) => build_footer_line(
            " FILTER ",
            Color::Magenta,
            FooterMode::NormalFilter,
            &app.keymap,
        ),
    }
}

fn footer_mode_to_context(mode: FooterMode) -> KeyContext {
    match mode {
        FooterMode::NormalDaily => KeyContext::DailyNormal,
        FooterMode::NormalFilter => KeyContext::FilterNormal,
        FooterMode::Edit => KeyContext::Edit,
        FooterMode::Reorder => KeyContext::Reorder,
        FooterMode::Selection => KeyContext::Selection,
        FooterMode::Datepicker => KeyContext::Datepicker,
    }
}

fn build_footer_line(
    mode_name: &str,
    color: Color,
    mode: FooterMode,
    keymap: &Keymap,
) -> RatatuiLine<'static> {
    let mut spans = vec![Span::styled(
        mode_name.to_string(),
        Style::default().fg(Color::Black).bg(color),
    )];

    let context = footer_mode_to_context(mode);

    for action in footer_actions(mode) {
        spans.extend(action_spans(action, keymap, context));
    }

    RatatuiLine::from(spans)
}

fn format_key_for_display(key: &str) -> String {
    match key {
        "down" => "↓".to_string(),
        "up" => "↑".to_string(),
        "left" => "←".to_string(),
        "right" => "→".to_string(),
        "ret" => "Enter".to_string(),
        "esc" => "Esc".to_string(),
        "tab" => "Tab".to_string(),
        "backtab" => "S-Tab".to_string(),
        "backspace" => "Bksp".to_string(),
        " " => "Space".to_string(),
        _ => key.to_string(),
    }
}

fn action_spans(action: &KeyAction, keymap: &Keymap, context: KeyContext) -> [Span<'static>; 2] {
    let keys = keymap.keys_for_action(context, action.id);

    let key_display = if keys.is_empty() {
        // Fall back to default_keys if no keys bound (shouldn't happen normally)
        match action.default_keys {
            [first, second, ..] => {
                format!(
                    "{}/{}",
                    format_key_for_display(first),
                    format_key_for_display(second)
                )
            }
            [first] => format_key_for_display(first),
            [] => String::new(),
        }
    } else if keys.len() == 1 {
        format_key_for_display(&keys[0])
    } else {
        format!(
            "{}/{}",
            format_key_for_display(&keys[0]),
            format_key_for_display(&keys[1])
        )
    };

    [
        Span::styled(format!("  {key_display}"), Style::default().fg(Color::Gray)),
        Span::styled(format!(" {} ", action.footer_text), Style::default().dim()),
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
