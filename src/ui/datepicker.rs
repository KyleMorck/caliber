use chrono::{Datelike, Local, NaiveDate};
use ratatui::widgets::calendar::{CalendarEventStore, Monthly};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};
use time::{Date, Month};

use crate::app::DatepickerState;
use crate::dispatch::Keymap;
use crate::registry::{FooterMode, KeyContext, footer_actions};

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

/// Convert chrono NaiveDate to time::Date (required by ratatui calendar)
fn to_time_date(date: NaiveDate) -> Date {
    Date::from_calendar_date(
        date.year(),
        Month::try_from(date.month() as u8).unwrap(),
        date.day() as u8,
    )
    .unwrap()
}

/// Create a fixed-size centered rect
fn centered_fixed_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;

    Rect {
        x,
        y,
        width: width.min(area.width),
        height: height.min(area.height),
    }
}

pub fn render_datepicker(f: &mut Frame, state: &DatepickerState, keymap: &Keymap, area: Rect) {
    // Fixed size popup - calendar is ~22 chars wide, we add padding
    let popup_width: u16 = 26;
    let popup_height: u16 = 11;

    let popup_area = centered_fixed_rect(popup_width, popup_height, area);

    f.render_widget(Clear, popup_area);

    let title = state.display_month.format(" %B %Y ").to_string();

    let block = Block::default()
        .title(Span::styled(title, Style::new().fg(Color::Cyan)))
        .borders(Borders::ALL)
        .border_style(Style::new().fg(Color::Cyan));

    let inner_area = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let today = Local::now().date_naive();
    let mut events = CalendarEventStore::default();

    // Style for days with content (not dimmed)
    // Priority: incomplete tasks (yellow) > events (blue) > content (gray)
    for (date, info) in &state.day_cache {
        if *date == state.selected || *date == today {
            continue;
        }
        if info.has_entries {
            let style = if info.has_incomplete_tasks {
                Style::new().fg(Color::Yellow).not_dim()
            } else if info.has_events {
                Style::new().fg(Color::Magenta).not_dim()
            } else {
                Style::new().fg(Color::Gray).not_dim()
            };
            events.add(to_time_date(*date), style);
        }
    }

    if today.month() == state.display_month.month()
        && today.year() == state.display_month.year()
        && today != state.selected
    {
        events.add(to_time_date(today), Style::new().fg(Color::Cyan).not_dim());
    }

    let selected_info = state.day_cache.get(&state.selected);
    let selected_style = if state.selected == today {
        Style::new().fg(Color::Cyan).reversed().not_dim()
    } else if selected_info
        .map(|i| i.has_incomplete_tasks)
        .unwrap_or(false)
    {
        Style::new().fg(Color::Yellow).reversed().not_dim()
    } else if selected_info.map(|i| i.has_events).unwrap_or(false) {
        Style::new().fg(Color::Magenta).reversed().not_dim()
    } else {
        Style::new().reversed().not_dim()
    };
    events.add(to_time_date(state.selected), selected_style);

    // Render calendar - default is dimmed Gray for empty days
    let calendar = Monthly::new(to_time_date(state.display_month), events)
        .show_weekdays_header(Style::new().fg(Color::Gray).dim().bold())
        .default_style(Style::new().fg(Color::Gray).dim());

    f.render_widget(calendar, inner_area);

    if inner_area.height >= 2 {
        let footer_area = Rect {
            x: inner_area.x,
            y: inner_area.y + inner_area.height.saturating_sub(1),
            width: inner_area.width,
            height: 1,
        };

        let mut spans: Vec<Span> = Vec::new();
        for action in footer_actions(FooterMode::Datepicker) {
            let keys = keymap.keys_for_action(KeyContext::Datepicker, action.id);
            let key_display = if keys.is_empty() {
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
            spans.push(Span::styled(
                format!(" {key_display}"),
                Style::new().fg(Color::Gray),
            ));
            spans.push(Span::styled(
                format!(" {}", action.footer_text),
                Style::new().dim(),
            ));
        }

        let footer = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);

        f.render_widget(footer, footer_area);
    }
}
