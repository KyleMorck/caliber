use ratatui::{
    style::{Style, Stylize},
    text::Span,
};
use unicode_width::UnicodeWidthStr;

use crate::app::{App, EditContext, InputMode, ViewMode};

use super::helpers::edit_text;
use super::list_helpers::{build_edit_rows, header_line};
use super::model::ListModel;
use super::rows;
use super::shared::{date_suffix_style, entry_style, format_date_suffix};
use super::theme;

pub fn build_filter_list(app: &App, width: usize) -> ListModel {
    let ViewMode::Filter(state) = &app.view else {
        return ListModel::from_rows(None, Vec::new(), app.scroll_offset());
    };

    let mut rows = Vec::new();

    let header = format!("Filter: {}", state.query);
    let header_line = header_line(header, Style::default().fg(theme::MODE_FILTER));

    let is_quick_adding = matches!(
        app.input_mode,
        InputMode::Edit(EditContext::FilterQuickAdd { .. })
    );
    let is_editing = matches!(
        app.input_mode,
        InputMode::Edit(EditContext::FilterEdit { .. })
    );

    for (idx, filter_entry) in state.entries.iter().enumerate() {
        let is_selected = idx == state.selected && !is_quick_adding;
        let is_editing_this = is_selected && is_editing;

        let content_style = entry_style(&filter_entry.entry_type);

        let text = edit_text(app, is_editing_this, &filter_entry.content);

        let prefix = filter_entry.entry_type.prefix();
        let prefix_width = prefix.width();

        if is_selected {
            if is_editing_this {
                let (date_suffix, date_suffix_width) = format_date_suffix(filter_entry.source_date);
                let text_width = width.saturating_sub(prefix_width + date_suffix_width);
                rows.extend(build_edit_rows(
                    &prefix.to_string(),
                    prefix_width,
                    content_style,
                    &text,
                    text_width,
                    Some(Span::styled(date_suffix, date_suffix_style(content_style))),
                ));
            } else {
                rows.push(rows::build_filter_selected_row(
                    app,
                    filter_entry,
                    idx,
                    width,
                ));
            }
        } else {
            rows.push(rows::build_filter_row(app, filter_entry, idx, width));
        }
    }

    if let InputMode::Edit(EditContext::FilterQuickAdd { entry_type, .. }) = &app.input_mode {
        let text = edit_text(app, true, "");
        let prefix = entry_type.prefix();
        let prefix_width = prefix.width();
        let text_width = width.saturating_sub(prefix_width);

        let content_style = entry_style(entry_type);
        rows.extend(build_edit_rows(
            &prefix.to_string(),
            prefix_width,
            content_style,
            &text,
            text_width,
            None,
        ));
    }

    if state.entries.is_empty() && !is_quick_adding {
        rows.push(rows::build_message_row(
            "(no matches)",
            Style::default().dim(),
        ));
    }

    ListModel::from_rows(Some(header_line), rows, app.scroll_offset())
}
