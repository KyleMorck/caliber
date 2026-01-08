use chrono::{Datelike, Local, NaiveDate};
use ratatui::widgets::calendar::{CalendarEventStore, Monthly};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::Span,
    widgets::{Block, Borders, Clear},
};
use time::{Date, Month};

use crate::app::DatepickerState;

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

pub fn render_datepicker(f: &mut Frame, state: &DatepickerState, area: Rect) {
    // Fixed size popup - calendar is 20 chars wide (Su Mo Tu We Th Fr Sa)
    let popup_width: u16 = 24;
    let popup_height: u16 = 10;

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
}
