use ratatui::{
    style::Style,
    text::{Line as RatatuiLine, Span},
};

use super::model::RowModel;
use super::rows;

pub fn header_line(text: impl Into<String>, style: Style) -> RatatuiLine<'static> {
    RatatuiLine::from(Span::styled(text.into(), style))
}

pub fn build_edit_rows(
    prefix: &str,
    prefix_width: usize,
    content_style: Style,
    text: &str,
    text_width: usize,
    suffix: Option<Span<'static>>,
) -> Vec<RowModel> {
    rows::build_edit_rows_with_prefix_width(
        prefix,
        prefix_width,
        content_style,
        text,
        text_width,
        suffix,
    )
}
