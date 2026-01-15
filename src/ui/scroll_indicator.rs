#![allow(dead_code)]

use super::theme;

pub enum ScrollIndicatorStyle {
    Arrows,
    Labeled,
}

pub fn scroll_indicator_text(
    can_scroll_up: bool,
    can_scroll_down: bool,
    style: ScrollIndicatorStyle,
) -> Option<String> {
    let arrows = match (can_scroll_up, can_scroll_down) {
        (true, true) => theme::GLYPH_SCROLL_BOTH,
        (true, false) => theme::GLYPH_SCROLL_UP,
        (false, true) => theme::GLYPH_SCROLL_DOWN,
        (false, false) => return None,
    };

    match style {
        ScrollIndicatorStyle::Arrows => Some(arrows.to_string()),
        ScrollIndicatorStyle::Labeled => Some(format!(
            "{label}{pad}{arrows}",
            pad = theme::SCROLL_PADDING,
            label = theme::SCROLL_LABEL
        )),
    }
}
