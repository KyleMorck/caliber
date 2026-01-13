use chrono::{Local, NaiveDate};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthStr;

use crate::app::App;
use crate::storage::{self, EntryType, SourceType};

use super::shared::truncate_text;
use super::theme;

pub struct AgendaDayModel {
    pub date: NaiveDate,
    pub entries: Vec<AgendaEntryModel>,
}

pub struct AgendaEntryModel {
    pub prefix: char,
    pub text: String,
    pub style: Style,
}

pub struct AgendaWidgetModel {
    pub days: Vec<AgendaDayModel>,
    pub width: usize,
}

impl AgendaWidgetModel {
    #[must_use]
    pub fn required_width(&self) -> usize {
        self.width
    }

    pub fn render_lines(&self) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        let content_width = self.width.saturating_sub(theme::AGENDA_BORDER_WIDTH);

        for (i, day) in self.days.iter().enumerate() {
            if i > 0 {
                lines.push(Line::from(""));
            }
            let date_str = day.date.format("%m/%d/%y").to_string();
            lines.push(Line::from(Span::styled(
                format!(" {date_str}"),
                Style::default().add_modifier(Modifier::DIM),
            )));

            for entry in &day.entries {
                let prefix = format!(" {} ", entry.prefix);
                let prefix_width = prefix.width();
                let max_text = content_width.saturating_sub(prefix_width);
                let text = truncate_text(&entry.text, max_text);
                lines.push(Line::from(vec![
                    Span::styled(prefix, entry.style),
                    Span::styled(text, entry.style),
                ]));
            }
        }

        lines
    }
}

pub fn build_agenda_widget(app: &App, width: usize, show_times: bool) -> AgendaWidgetModel {
    let today = Local::now().date_naive();
    let path = app.active_path();
    let mut days = Vec::new();
    let mut max_width = theme::AGENDA_DATE_WIDTH;
    let mut total_entries = 0usize;

    for day_offset in 0..theme::AGENDA_MAX_DAYS_SEARCH {
        let date = today + chrono::Duration::days(day_offset);
        let mut entries = Vec::new();

        for event in app.calendar_store.events_for_date(date) {
            let text = if !show_times || event.is_all_day {
                event.title.clone()
            } else {
                format!("{} {}", event.start.format("%l:%M%P"), event.title)
            };
            max_width = max_width.max(text.width() + theme::AGENDA_ENTRY_PADDING);
            entries.push(AgendaEntryModel {
                prefix: theme::GLYPH_AGENDA_CALENDAR,
                text,
                style: Style::default(),
            });
        }

        if let Ok(projected) = storage::collect_projected_entries_for_date(date, path) {
            for entry in projected.iter() {
                let is_recurring = entry.source_type == SourceType::Recurring;
                let is_event = entry.entry_type == EntryType::Event;
                if !is_recurring && !is_event {
                    continue;
                }
                let (prefix, style) =
                    projected_entry_prefix_and_style(&entry.source_type, &entry.entry_type);
                let text = truncate_to_first_tag(&entry.content);
                max_width = max_width.max(text.width() + theme::AGENDA_ENTRY_PADDING);
                entries.push(AgendaEntryModel {
                    prefix,
                    text,
                    style,
                });
            }
        }

        if let Ok(day_lines) = storage::load_day_lines(date, path) {
            for line in &day_lines {
                if let storage::Line::Entry(raw) = line {
                    if raw.entry_type != EntryType::Event {
                        continue;
                    }
                    let text = truncate_to_first_tag(&raw.content);
                    max_width = max_width.max(text.width() + theme::AGENDA_ENTRY_PADDING);
                    entries.push(AgendaEntryModel {
                        prefix: theme::GLYPH_AGENDA_EVENT,
                        text,
                        style: Style::default().add_modifier(Modifier::ITALIC),
                    });
                }
            }
        }

        if !entries.is_empty() {
            total_entries += entries.len();
            days.push(AgendaDayModel { date, entries });
        }

        if total_entries >= theme::AGENDA_MIN_ENTRIES {
            break;
        }
    }

    AgendaWidgetModel {
        days,
        width: width.min(max_width),
    }
}

fn projected_entry_prefix_and_style(
    source_type: &SourceType,
    entry_type: &EntryType,
) -> (char, Style) {
    if *source_type == SourceType::Recurring {
        return (theme::GLYPH_AGENDA_RECURRING, Style::default());
    }
    if *entry_type == EntryType::Event {
        return (
            theme::GLYPH_AGENDA_EVENT,
            Style::default().add_modifier(Modifier::ITALIC),
        );
    }
    (theme::GLYPH_AGENDA_FALLBACK, Style::default())
}

fn truncate_to_first_tag(content: &str) -> String {
    if let Some(pos) = content.find('#') {
        content[..pos].trim().to_string()
    } else {
        content.to_string()
    }
}
