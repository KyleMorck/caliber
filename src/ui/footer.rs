#![allow(dead_code)]

use ratatui::text::Line as RatatuiLine;

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
