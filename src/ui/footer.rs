use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::Line as RatatuiLine,
};

use crate::app::{InputMode, ViewMode};
use crate::dispatch::Keymap;

pub struct FooterModel<'a> {
    pub view: &'a ViewMode,
    pub input_mode: &'a InputMode,
    pub keymap: &'a Keymap,
}

impl<'a> FooterModel<'a> {
    #[must_use]
    pub fn new(view: &'a ViewMode, input_mode: &'a InputMode, keymap: &'a Keymap) -> Self {
        Self {
            view,
            input_mode,
            keymap,
        }
    }
}

pub fn render_footer(_model: FooterModel<'_>) -> RatatuiLine<'static> {
    RatatuiLine::from("")
}

pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}
