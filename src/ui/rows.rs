use chrono::Timelike;
use ratatui::{
    style::{Color, Style, Stylize},
    text::Span,
};
use unicode_width::UnicodeWidthStr;

use crate::app::{App, InputMode};
use crate::calendar::CalendarEvent;
use crate::storage::{Entry, EntryType, RawEntry, SourceType};

use super::model::RowModel;
use super::shared::{
    date_suffix_style, entry_style, format_date_suffix, style_content, truncate_with_tags,
    wrap_text,
};
use super::theme;

pub fn build_calendar_row(
    event: &CalendarEvent,
    width: usize,
    show_calendar_name: bool,
) -> RowModel {
    let prefix = "* ";
    let prefix_width = prefix.width();
    let indicator = "○";

    let content = format_calendar_event(event, show_calendar_name);
    let available = width.saturating_sub(prefix_width);
    let display_text = truncate_with_tags(&content, available);

    let content_style = if event.is_cancelled || event.is_declined {
        Style::default().italic().crossed_out()
    } else {
        Style::default().italic()
    };
    let (_, rest_of_prefix) = split_prefix(prefix);

    RowModel::new(
        Some(Span::styled(
            indicator.to_string(),
            Style::default().fg(theme::CALENDAR_INDICATOR),
        )),
        Some(Span::styled(rest_of_prefix, content_style)),
        style_content(&display_text, content_style),
        None,
    )
}

pub fn build_projected_row(
    app: &App,
    projected_entry: &Entry,
    is_selected: bool,
    visible_idx: usize,
    width: usize,
) -> RowModel {
    let content_style = entry_style(&projected_entry.entry_type);
    let text = projected_entry.content.clone();
    let prefix = projected_entry.entry_type.prefix();
    let prefix_width = prefix.width();
    let (source_suffix, source_suffix_width) = format_date_suffix(projected_entry.source_date);

    let available = width.saturating_sub(prefix_width + source_suffix_width);
    let display_text = truncate_with_tags(&text, available);
    let (_, rest_of_prefix) = split_prefix(prefix);

    let indicator =
        get_projected_entry_indicator(app, is_selected, visible_idx, &projected_entry.source_type);

    RowModel::new(
        Some(indicator),
        Some(Span::styled(rest_of_prefix, content_style)),
        style_content(&display_text, content_style),
        Some(Span::styled(
            source_suffix,
            date_suffix_style(content_style),
        )),
    )
}

pub fn build_daily_entry_row(
    app: &App,
    entry: &RawEntry,
    is_selected: bool,
    visible_idx: usize,
    width: usize,
) -> RowModel {
    let content_style = entry_style(&entry.entry_type);
    let text = entry.content.clone();
    let prefix = entry.prefix();
    let prefix_width = prefix.width();

    let (first_char, rest_of_prefix) = split_prefix(&prefix);
    let indicator = get_entry_indicator(
        app,
        is_selected,
        visible_idx,
        theme::ENTRY_CURSOR,
        &first_char,
        content_style,
    );
    let available = width.saturating_sub(prefix_width);
    let display_text = truncate_with_tags(&text, available);

    RowModel::new(
        Some(indicator),
        Some(Span::styled(rest_of_prefix, content_style)),
        style_content(&display_text, content_style),
        None,
    )
}

pub fn build_filter_selected_row(app: &App, entry: &Entry, index: usize, width: usize) -> RowModel {
    let content_style = entry_style(&entry.entry_type);
    let text = entry.content.clone();
    let prefix = entry.entry_type.prefix();
    let prefix_width = prefix.width();
    let (date_suffix, date_suffix_width) = format_date_suffix(entry.source_date);

    let sel_prefix = match &entry.entry_type {
        EntryType::Task { completed: false } => " [ ] ",
        EntryType::Task { completed: true } => " [x] ",
        EntryType::Note => " ",
        EntryType::Event => " ",
    };
    let available = width.saturating_sub(prefix_width + date_suffix_width);
    let display_text = truncate_with_tags(&text, available);

    RowModel::new(
        Some(filter_cursor_indicator(app, index)),
        Some(Span::styled(sel_prefix.to_string(), content_style)),
        style_content(&display_text, content_style),
        Some(Span::styled(date_suffix, date_suffix_style(content_style))),
    )
}

pub fn build_filter_row(app: &App, entry: &Entry, index: usize, width: usize) -> RowModel {
    let content_style = entry_style(&entry.entry_type);
    let text = entry.content.clone();
    let prefix = entry.entry_type.prefix();
    let prefix_width = prefix.width();
    let (date_suffix, date_suffix_width) = format_date_suffix(entry.source_date);

    let available = width.saturating_sub(prefix_width + date_suffix_width);
    let display_text = truncate_with_tags(&text, available);

    let (first_char, rest_of_prefix) = split_prefix(&prefix);
    let first_char = filter_list_indicator(app, &first_char, index, content_style);

    RowModel::new(
        Some(first_char),
        Some(Span::styled(rest_of_prefix, content_style)),
        style_content(&display_text, content_style),
        Some(Span::styled(date_suffix, date_suffix_style(content_style))),
    )
}

pub fn build_edit_rows_with_prefix_width(
    prefix: &str,
    prefix_width: usize,
    content_style: Style,
    text: &str,
    text_width: usize,
    suffix: Option<Span<'static>>,
) -> Vec<RowModel> {
    let wrapped = wrap_text(text, text_width);

    if wrapped.is_empty() {
        return vec![RowModel::new(
            None,
            Some(Span::styled(prefix.to_string(), content_style)),
            Vec::new(),
            suffix,
        )];
    }

    wrapped
        .iter()
        .enumerate()
        .map(|(i, line_text)| {
            let prefix_text = if i == 0 {
                prefix.to_string()
            } else {
                " ".repeat(prefix_width)
            };
            RowModel::new(
                None,
                Some(Span::styled(prefix_text, content_style)),
                style_content(line_text, content_style),
                if i == 0 { suffix.clone() } else { None },
            )
        })
        .collect()
}

pub fn build_message_row(message: &str, style: Style) -> RowModel {
    RowModel::from_spans(vec![Span::styled(message.to_string(), style)])
}

fn split_prefix(prefix: &str) -> (String, String) {
    let mut chars = prefix.chars();
    let first_char = chars.next().unwrap_or('-');
    let rest: String = chars.collect();
    (first_char.to_string(), rest)
}

fn is_selected_in_selection(app: &App, index: usize) -> bool {
    if let InputMode::Selection(ref state) = app.input_mode {
        state.is_selected(index)
    } else {
        false
    }
}

fn filter_cursor_indicator(app: &App, index: usize) -> Span<'static> {
    if is_selected_in_selection(app, index) {
        Span::styled("◉", Style::default().fg(theme::ENTRY_SELECTION))
    } else {
        Span::styled("→", Style::default().fg(theme::ENTRY_CURSOR))
    }
}

fn filter_list_indicator(
    app: &App,
    first_char: &str,
    index: usize,
    content_style: Style,
) -> Span<'static> {
    if is_selected_in_selection(app, index) {
        Span::styled("○", Style::default().fg(theme::ENTRY_SELECTION))
    } else {
        Span::styled(first_char.to_string(), content_style)
    }
}

fn format_calendar_event(event: &CalendarEvent, show_calendar_name: bool) -> String {
    let mut parts = vec![event.title.clone()];

    if let Some((day, total)) = event.multi_day_info {
        parts.push(format!("{day}/{total}"));
    }

    if !event.is_all_day {
        let start_hour = event.start.hour();
        let end_hour = event.end.hour();
        let same_period = (start_hour < 12) == (end_hour < 12);

        let time_str = if same_period {
            let start_time = event.start.format("%-I:%M").to_string();
            let end_time = event.end.format("%-I:%M%P").to_string();
            format!("{start_time}-{end_time}")
        } else {
            let start_time = event.start.format("%-I:%M%P").to_string();
            let end_time = event.end.format("%-I:%M%P").to_string();
            format!("{start_time}-{end_time}")
        };
        parts.push(time_str);
    }

    if show_calendar_name {
        parts.push(format!("({})", event.calendar_name));
    }

    if parts.len() == 1 {
        parts.into_iter().next().unwrap()
    } else if show_calendar_name && parts.len() > 1 {
        let last = parts.pop().unwrap();
        format!("{} {last}", parts.join(" - "))
    } else {
        parts.join(" - ")
    }
}

fn get_projected_entry_indicator(
    _app: &App,
    is_cursor: bool,
    _visible_idx: usize,
    kind: &SourceType,
) -> Span<'static> {
    let indicator = match kind {
        SourceType::Later => "↪",
        SourceType::Recurring => "↺",
        SourceType::Local => unreachable!("projected entries are never Local"),
        SourceType::Calendar { .. } => "○",
    };

    if is_cursor {
        Span::styled(
            indicator,
            Style::default().fg(theme::ENTRY_PROJECTED_ACTIVE),
        )
    } else {
        Span::styled(
            indicator,
            Style::default().fg(theme::ENTRY_PROJECTED_INACTIVE),
        )
    }
}

fn get_entry_indicator(
    app: &App,
    is_cursor: bool,
    visible_idx: usize,
    cursor_color: Color,
    default_first_char: &str,
    default_style: Style,
) -> Span<'static> {
    let is_selected_in_selection = is_selected_in_selection(app, visible_idx);

    if is_cursor {
        if matches!(app.input_mode, InputMode::Reorder) {
            Span::styled("↕", Style::default().fg(theme::ENTRY_SELECTION))
        } else if matches!(app.input_mode, InputMode::Selection(_)) {
            if is_selected_in_selection {
                Span::styled("◉", Style::default().fg(theme::ENTRY_SELECTION))
            } else {
                Span::styled("→", Style::default().fg(theme::ENTRY_CURSOR))
            }
        } else {
            Span::styled("→", Style::default().fg(cursor_color))
        }
    } else if is_selected_in_selection {
        Span::styled("○", Style::default().fg(theme::ENTRY_SELECTION))
    } else {
        Span::styled(default_first_char.to_string(), default_style)
    }
}
