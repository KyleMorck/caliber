pub enum ScrollIndicatorStyle {
    Arrows,
    Labeled,
}

pub fn scroll_indicator_text(
    can_scroll_up: bool,
    can_scroll_down: bool,
    style: ScrollIndicatorStyle,
) -> Option<&'static str> {
    match (can_scroll_up, can_scroll_down, style) {
        (true, true, ScrollIndicatorStyle::Arrows) => Some("▲▼"),
        (true, false, ScrollIndicatorStyle::Arrows) => Some("▲"),
        (false, true, ScrollIndicatorStyle::Arrows) => Some("▼"),
        (true, true, ScrollIndicatorStyle::Labeled) => Some(" ▲▼ scroll "),
        (true, false, ScrollIndicatorStyle::Labeled) => Some(" ▲ scroll "),
        (false, true, ScrollIndicatorStyle::Labeled) => Some(" ▼ scroll "),
        (false, false, _) => None,
    }
}
