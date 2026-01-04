use ratatui::{
    style::{Color, Style},
    text::{Line as RatatuiLine, Span},
};
use unicode_width::UnicodeWidthStr;

use crate::app::{App, EditContext, InputMode, ViewMode};
use crate::storage::{EntryType, Line};

use super::shared::{style_content, truncate_with_tags, wrap_text};

pub fn render_daily_view(app: &App, width: usize) -> Vec<RatatuiLine<'static>> {
    let ViewMode::Daily(state) = &app.view else {
        return vec![];
    };

    let mut lines = Vec::new();

    let date_header = app.current_date.format("%m/%d/%y").to_string();
    let hidden_count = app.hidden_completed_count();
    if app.hide_completed && hidden_count > 0 {
        lines.push(RatatuiLine::from(vec![
            Span::styled(date_header, Style::default().fg(Color::Cyan)),
            Span::styled(
                format!(" (Hiding {hidden_count} completed)"),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    } else {
        lines.push(RatatuiLine::from(Span::styled(
            date_header,
            Style::default().fg(Color::Cyan),
        )));
    }

    let mut visible_later_idx = 0;

    // === Later entries section (at top) ===
    for later_entry in &state.later_entries {
        if app.hide_completed && later_entry.completed {
            continue;
        }

        let is_selected = visible_later_idx == state.selected;
        visible_later_idx += 1;
        let is_editing = is_selected
            && matches!(
                app.input_mode,
                InputMode::Edit(EditContext::LaterEdit { .. })
            );

        let content_style = if later_entry.completed {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default()
        };

        let text = if is_editing {
            if let Some(ref buffer) = app.edit_buffer {
                buffer.content().to_string()
            } else {
                later_entry.content.clone()
            }
        } else {
            later_entry.content.clone()
        };

        let prefix = later_entry.entry_type.prefix();
        let prefix_width = prefix.width();
        let source_suffix = format!(" ({})", later_entry.source_date.format("%m/%d"));
        let source_suffix_width = source_suffix.width();
        let later_prefix_style = Style::default().fg(Color::Red);

        if is_editing {
            let available = width.saturating_sub(prefix_width + source_suffix_width);
            let wrapped = wrap_text(&text, available);
            for (i, line_text) in wrapped.iter().enumerate() {
                if i == 0 {
                    let first_char = prefix.chars().next().unwrap_or('-').to_string();
                    let rest_of_prefix: String = prefix.chars().skip(1).collect();
                    let spans = vec![
                        Span::styled(first_char, later_prefix_style),
                        Span::styled(rest_of_prefix, content_style),
                        Span::styled(line_text.clone(), content_style),
                        Span::styled(source_suffix.clone(), Style::default().fg(Color::DarkGray)),
                    ];
                    lines.push(RatatuiLine::from(spans));
                } else {
                    let indent = " ".repeat(prefix_width);
                    lines.push(RatatuiLine::from(Span::styled(
                        format!("{indent}{line_text}"),
                        content_style,
                    )));
                }
            }
        } else if is_selected {
            let available = width.saturating_sub(prefix_width + source_suffix_width);
            let display_text = truncate_with_tags(&text, available);
            let rest_of_prefix: String = prefix.chars().skip(1).collect();
            let mut spans = vec![
                Span::styled("→", Style::default().fg(Color::Red)),
                Span::styled(rest_of_prefix, content_style),
            ];
            spans.extend(style_content(
                &display_text,
                content_style,
                later_entry.completed,
            ));
            spans.push(Span::styled(
                source_suffix,
                Style::default().fg(Color::DarkGray),
            ));
            lines.push(RatatuiLine::from(spans));
        } else {
            let available = width.saturating_sub(prefix_width + source_suffix_width);
            let display_text = truncate_with_tags(&text, available);
            let first_char = prefix.chars().next().unwrap_or('-').to_string();
            let rest_of_prefix: String = prefix.chars().skip(1).collect();
            let mut spans = vec![
                Span::styled(first_char, later_prefix_style),
                Span::styled(rest_of_prefix, content_style),
            ];
            spans.extend(style_content(
                &display_text,
                content_style,
                later_entry.completed,
            ));
            spans.push(Span::styled(
                source_suffix,
                Style::default().fg(Color::DarkGray),
            ));
            lines.push(RatatuiLine::from(spans));
        }
    }

    // === Regular entries section ===
    let mut visible_entry_idx = 0;
    for &line_idx in &app.entry_indices {
        if let Line::Entry(entry) = &app.lines[line_idx] {
            let is_completed = matches!(entry.entry_type, EntryType::Task { completed: true });

            if app.hide_completed && is_completed {
                continue;
            }

            let selection_idx = visible_later_idx + visible_entry_idx;
            visible_entry_idx += 1;
            let is_selected = selection_idx == state.selected;
            let is_editing =
                is_selected && matches!(app.input_mode, InputMode::Edit(EditContext::Daily { .. }));

            let content_style = if is_completed {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default()
            };

            let text = if is_editing {
                if let Some(ref buffer) = app.edit_buffer {
                    buffer.content().to_string()
                } else {
                    entry.content.clone()
                }
            } else {
                entry.content.clone()
            };

            let prefix = entry.prefix();
            let prefix_width = prefix.width();

            if is_editing {
                let wrapped = wrap_text(&text, width.saturating_sub(prefix_width));
                for (i, line_text) in wrapped.iter().enumerate() {
                    if i == 0 {
                        lines.push(RatatuiLine::from(Span::styled(
                            format!("{prefix}{line_text}"),
                            content_style,
                        )));
                    } else {
                        let indent = " ".repeat(prefix_width);
                        lines.push(RatatuiLine::from(Span::styled(
                            format!("{indent}{line_text}"),
                            content_style,
                        )));
                    }
                }
            } else if is_selected {
                let rest_of_prefix = prefix.chars().skip(1).collect::<String>();
                let indicator = if app.input_mode == InputMode::Reorder {
                    Span::styled("↕", Style::default().fg(Color::Yellow))
                } else {
                    Span::styled("→", Style::default().fg(Color::Cyan))
                };
                let available = width.saturating_sub(prefix_width);
                let display_text = truncate_with_tags(&text, available);
                let mut spans = vec![indicator, Span::styled(rest_of_prefix, content_style)];
                spans.extend(style_content(&display_text, content_style, is_completed));
                lines.push(RatatuiLine::from(spans));
            } else {
                let available = width.saturating_sub(prefix_width);
                let display_text = truncate_with_tags(&text, available);
                let mut spans = vec![Span::styled(prefix.to_string(), content_style)];
                spans.extend(style_content(&display_text, content_style, is_completed));
                lines.push(RatatuiLine::from(spans));
            }
        }
    }

    if visible_later_idx == 0 && visible_entry_idx == 0 {
        let has_hidden = app.hide_completed && app.hidden_completed_count() > 0;
        let message = if has_hidden {
            "(No visible entries - press z to show completed or Enter to add)"
        } else {
            "(No entries - press Enter to add)"
        };
        lines.push(RatatuiLine::from(Span::styled(
            message,
            Style::default().fg(Color::DarkGray),
        )));
    }

    lines
}
