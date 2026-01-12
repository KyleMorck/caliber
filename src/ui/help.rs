use ratatui::text::Line as RatatuiLine;

use crate::dispatch::Keymap;

#[must_use]
pub fn get_help_total_lines(_keymap: &Keymap) -> usize {
    0
}

pub fn render_help_content(
    _keymap: &Keymap,
    _scroll: usize,
    _visible_height: usize,
) -> Vec<RatatuiLine<'static>> {
    Vec::new()
}
