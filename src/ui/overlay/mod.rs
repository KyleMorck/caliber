use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Style, Stylize},
    text::{Line as RatatuiLine, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::ConfirmContext;

use super::footer::centered_rect;
use super::theme;

pub struct OverlayModel<'a> {
    pub confirm: Option<ConfirmModel<'a>>,
}

pub struct OverlayLayout {
    pub screen_area: Rect,
}

pub struct ConfirmModel<'a> {
    pub context: &'a ConfirmContext,
}

impl<'a> ConfirmModel<'a> {
    #[must_use]
    pub fn new(context: &'a ConfirmContext) -> Self {
        Self { context }
    }
}

pub fn render_confirm_modal(f: &mut Frame<'_>, model: ConfirmModel<'_>, area: Rect) {
    let (title, messages): (&str, Vec<String>) = match model.context {
        ConfirmContext::CreateProjectJournal => (
            " Create Project Journal ",
            vec![
                "No project journal found.".to_string(),
                "Create .caliber/journal.md?".to_string(),
            ],
        ),
        ConfirmContext::DeleteTag(tag) => (
            " Delete Tag ",
            vec![
                format!("Delete all occurrences of #{}?", tag),
                "This cannot be undone.".to_string(),
            ],
        ),
    };

    let popup_area = centered_rect(50, 30, area);
    f.render_widget(Clear, popup_area);

    let confirm_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::CONFIRM_BORDER));

    let inner_area = confirm_block.inner(popup_area);
    f.render_widget(confirm_block, popup_area);

    let mut lines = vec![RatatuiLine::raw("")];
    for msg in messages {
        lines.push(RatatuiLine::raw(msg));
    }
    lines.push(RatatuiLine::raw(""));
    lines.push(RatatuiLine::from(vec![
        Span::styled("[Y]", Style::default().fg(theme::CONFIRM_YES)),
        Span::raw(" Yes    "),
        Span::styled("[N]", Style::default().fg(theme::CONFIRM_NO)),
        Span::raw(" No"),
    ]));
    let content = ratatui::text::Text::from(lines);
    let paragraph = Paragraph::new(content).alignment(Alignment::Center);
    f.render_widget(paragraph, inner_area);
}

pub fn render_overlays(f: &mut Frame<'_>, overlays: OverlayModel<'_>, layout: OverlayLayout) {
    if let Some(confirm) = overlays.confirm {
        render_confirm_modal(f, confirm, layout.screen_area);
    }
}
