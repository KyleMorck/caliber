use std::io;

use crate::app::{App, EntryLocation, ViewMode};
use crate::storage::{self, Line};

/// Target for content operations - includes original content for undo.
/// Used as a base type for TagTarget and DateTarget.
#[derive(Clone)]
pub struct ContentTarget {
    pub location: EntryLocation,
    pub original_content: String,
}

impl ContentTarget {
    #[must_use]
    pub fn new(location: EntryLocation, original_content: String) -> Self {
        Self {
            location,
            original_content,
        }
    }
}

/// Get entry content at a location
pub fn get_entry_content(app: &App, location: &EntryLocation) -> io::Result<String> {
    match location {
        EntryLocation::Projected(entry) => {
            let lines = storage::load_day_lines(entry.source_date, app.active_path())?;
            if let Some(Line::Entry(raw_entry)) = lines.get(entry.line_index) {
                Ok(raw_entry.content.clone())
            } else {
                Ok(String::new())
            }
        }
        EntryLocation::Daily { line_idx } => {
            if let Line::Entry(raw_entry) = &app.lines[*line_idx] {
                Ok(raw_entry.content.clone())
            } else {
                Ok(String::new())
            }
        }
        EntryLocation::Filter { entry, .. } => {
            let lines = storage::load_day_lines(entry.source_date, app.active_path())?;
            if let Some(Line::Entry(raw_entry)) = lines.get(entry.line_index) {
                Ok(raw_entry.content.clone())
            } else {
                Ok(String::new())
            }
        }
    }
}

/// Set entry content directly (for undo/restore operations)
pub fn set_entry_content(app: &mut App, location: &EntryLocation, content: &str) -> io::Result<()> {
    let path = app.active_path().to_path_buf();

    match location {
        EntryLocation::Projected(entry) => {
            storage::mutate_entry(entry.source_date, &path, entry.line_index, |raw_entry| {
                raw_entry.content = content.to_string();
            })?;

            app.refresh_projected_entries();
        }
        EntryLocation::Daily { line_idx } => {
            if let Line::Entry(raw_entry) = &mut app.lines[*line_idx] {
                raw_entry.content = content.to_string();
                app.save();
            }
        }
        EntryLocation::Filter { index, entry } => {
            storage::mutate_entry(entry.source_date, &path, entry.line_index, |raw_entry| {
                raw_entry.content = content.to_string();
            })?;

            if let ViewMode::Filter(state) = &mut app.view
                && let Some(filter_entry) = state.entries.get_mut(*index)
            {
                filter_entry.content = content.to_string();
            }

            if entry.source_date == app.current_date {
                app.reload_current_day()?;
            }
        }
    }
    Ok(())
}

/// Execute a content transformation on a single target.
/// The operation function receives the current content and returns Some(new_content) if modified.
pub fn execute_content_operation<F>(
    app: &mut App,
    location: &EntryLocation,
    operation: F,
) -> io::Result<()>
where
    F: Fn(&str) -> Option<String>,
{
    let path = app.active_path().to_path_buf();

    match location {
        EntryLocation::Projected(entry) => {
            let changed =
                storage::mutate_entry(entry.source_date, &path, entry.line_index, |raw_entry| {
                    if let Some(new_content) = operation(&raw_entry.content) {
                        raw_entry.content = new_content;
                        true
                    } else {
                        false
                    }
                })?;

            if changed == Some(true) {
                app.refresh_projected_entries();
            }
        }
        EntryLocation::Daily { line_idx } => {
            if let Line::Entry(raw_entry) = &mut app.lines[*line_idx]
                && let Some(new_content) = operation(&raw_entry.content)
            {
                raw_entry.content = new_content;
                app.save();
            }
        }
        EntryLocation::Filter { index, entry } => {
            let new_content =
                storage::mutate_entry(entry.source_date, &path, entry.line_index, |raw_entry| {
                    if let Some(new_content) = operation(&raw_entry.content) {
                        raw_entry.content = new_content.clone();
                        Some(new_content)
                    } else {
                        None
                    }
                })?;

            if let Some(Some(content)) = new_content {
                if let ViewMode::Filter(state) = &mut app.view
                    && let Some(filter_entry) = state.entries.get_mut(*index)
                {
                    filter_entry.content = content;
                }

                if entry.source_date == app.current_date {
                    app.reload_current_day()?;
                }
            }
        }
    }
    Ok(())
}

/// Execute an unconditional content append on a single target.
pub fn execute_content_append(
    app: &mut App,
    location: &EntryLocation,
    suffix: &str,
) -> io::Result<()> {
    let path = app.active_path().to_path_buf();

    match location {
        EntryLocation::Projected(entry) => {
            storage::mutate_entry(entry.source_date, &path, entry.line_index, |raw_entry| {
                raw_entry.content.push_str(suffix);
            })?;

            app.refresh_projected_entries();
        }
        EntryLocation::Daily { line_idx } => {
            if let Line::Entry(raw_entry) = &mut app.lines[*line_idx] {
                raw_entry.content.push_str(suffix);
                app.save();
            }
        }
        EntryLocation::Filter { index, entry } => {
            storage::mutate_entry(entry.source_date, &path, entry.line_index, |raw_entry| {
                raw_entry.content.push_str(suffix);
            })?;

            if let ViewMode::Filter(state) = &mut app.view
                && let Some(filter_entry) = state.entries.get_mut(*index)
            {
                filter_entry.content.push_str(suffix);
            }

            if entry.source_date == app.current_date {
                app.reload_current_day()?;
            }
        }
    }
    Ok(())
}
