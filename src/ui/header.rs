use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    text::{Line as RatatuiLine, Span},
    widgets::Paragraph,
};

pub struct HeaderModel {
    pub left: Option<RatatuiLine<'static>>,
    pub right: Option<RatatuiLine<'static>>,
}

impl HeaderModel {
    #[must_use]
    pub fn new() -> Self {
        Self {
            left: None,
            right: None,
        }
    }
}

pub fn render_header_bar(f: &mut Frame<'_>, area: Rect, model: HeaderModel) {
    if model.left.is_none() && model.right.is_none() {
        return;
    }

    let left = model
        .left
        .unwrap_or_else(|| RatatuiLine::from(Span::raw("")));
    let right = model
        .right
        .unwrap_or_else(|| RatatuiLine::from(Span::raw("")));

    let left_paragraph = Paragraph::new(left);
    let right_paragraph = Paragraph::new(right).alignment(Alignment::Right);

    f.render_widget(left_paragraph, area);
    f.render_widget(right_paragraph, area);
}
