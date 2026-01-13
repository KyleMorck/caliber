use crate::app::{App, DATE_SUFFIX_WIDTH, EditContext, FILTER_HEADER_LINES, InputMode, ViewMode};
use crate::cursor::cursor_position_in_wrap;
use crate::storage::Line;
use unicode_width::UnicodeWidthStr;

use super::context::RenderContext;
use super::scroll::{CursorContext, ensure_selected_visible};
use super::views::{
    list_content_height_for_daily, list_content_height_for_filter, list_content_width_for_daily,
    list_content_width_for_filter,
};

const DAILY_LIST_HEADER_LINES: usize = 0;

pub struct RenderPrep {
    pub edit_cursor: Option<CursorContext>,
}

fn daily_fixed_lines(app: &App) -> usize {
    DAILY_LIST_HEADER_LINES + app.calendar_event_count()
}

/// Prepares render state and mutates view scroll offsets for visibility.
pub fn prepare_render(app: &mut App, layout: &RenderContext) -> RenderPrep {
    let filter_visual_line = app.filter_visual_line();
    let filter_total_lines = app.filter_total_lines();
    let visible_entry_count = app.visible_entry_count();
    let daily_fixed_lines = daily_fixed_lines(app);

    match &mut app.view {
        ViewMode::Filter(state) => {
            let scroll_height = list_content_height_for_filter(layout);
            ensure_selected_visible(
                &mut state.scroll_offset,
                filter_visual_line,
                filter_total_lines,
                scroll_height,
            );
            if state.selected == 0 {
                state.scroll_offset = 0;
            }
        }
        ViewMode::Daily(state) => {
            let scroll_height = list_content_height_for_daily(layout);
            ensure_selected_visible(
                &mut state.scroll_offset,
                state.selected + daily_fixed_lines,
                visible_entry_count + daily_fixed_lines,
                scroll_height,
            );
            if state.selected == 0 {
                state.scroll_offset = 0;
            }
        }
    }

    let edit_cursor = if let InputMode::Edit(ref ctx) = app.input_mode
        && let Some(ref buffer) = app.edit_buffer
    {
        match ctx {
            EditContext::FilterQuickAdd { entry_type, .. } => {
                let ViewMode::Filter(state) = &app.view else {
                    unreachable!()
                };
                let prefix_width = entry_type.prefix().len();
                let text_width = list_content_width_for_filter(layout).saturating_sub(prefix_width);
                let wrap_width = text_width.saturating_sub(1).max(1);
                let (cursor_row, cursor_col) = cursor_position_in_wrap(
                    buffer.content(),
                    buffer.cursor_display_pos(),
                    wrap_width,
                );
                Some(CursorContext {
                    prefix_width,
                    cursor_row,
                    cursor_col,
                    entry_start_line: state.entries.len() + FILTER_HEADER_LINES,
                })
            }
            EditContext::FilterEdit { filter_index, .. } => {
                let ViewMode::Filter(state) = &app.view else {
                    unreachable!()
                };
                state.entries.get(*filter_index).map(|filter_entry| {
                    let prefix_width = filter_entry.entry_type.prefix().len();
                    let text_width = list_content_width_for_filter(layout)
                        .saturating_sub(prefix_width + DATE_SUFFIX_WIDTH);
                    let wrap_width = text_width.saturating_sub(1).max(1);
                    let (cursor_row, cursor_col) = cursor_position_in_wrap(
                        buffer.content(),
                        buffer.cursor_display_pos(),
                        wrap_width,
                    );
                    CursorContext {
                        prefix_width,
                        cursor_row,
                        cursor_col,
                        entry_start_line: *filter_index + FILTER_HEADER_LINES,
                    }
                })
            }
            EditContext::Daily { entry_index } => app
                .entry_indices
                .get(*entry_index)
                .and_then(|&i| {
                    if let Line::Entry(entry) = &app.lines[i] {
                        Some(&entry.entry_type)
                    } else {
                        None
                    }
                })
                .map(|entry_type| {
                    let list_content_width = list_content_width_for_daily(layout);

                    let prefix_width = entry_type.prefix().width();
                    let text_width = list_content_width.saturating_sub(prefix_width);
                    let wrap_width = text_width.saturating_sub(1).max(1);

                    let (cursor_row, cursor_col) = cursor_position_in_wrap(
                        buffer.content(),
                        buffer.cursor_display_pos(),
                        wrap_width,
                    );
                    CursorContext {
                        prefix_width,
                        cursor_row,
                        cursor_col,
                        entry_start_line: daily_fixed_lines
                            + app.visible_projected_count()
                            + app.visible_entries_before(*entry_index),
                    }
                }),
        }
    } else {
        None
    };

    RenderPrep { edit_cursor }
}
