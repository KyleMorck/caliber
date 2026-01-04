use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::HintContext;

const HINT_OVERLAY_HEIGHT: u16 = 4;
const COLUMN_WIDTH: usize = 16;
const MAX_COLUMNS: usize = 4;
const MAX_ITEMS: usize = 16;

/// Renders the hint overlay above the footer.
/// Returns true if overlay was rendered.
pub fn render_hint_overlay(f: &mut Frame, hint_state: &HintContext, footer_area: Rect) -> bool {
    if matches!(hint_state, HintContext::Inactive) {
        return false;
    }

    // Calculate overlay area above the footer
    let overlay_area = Rect {
        x: footer_area.x,
        y: footer_area.y.saturating_sub(HINT_OVERLAY_HEIGHT),
        width: footer_area.width,
        height: HINT_OVERLAY_HEIGHT,
    };

    // Don't render if not enough space
    if overlay_area.height == 0 || overlay_area.width < 20 {
        return false;
    }

    // Clear the area first
    f.render_widget(Clear, overlay_area);

    let block = Block::default()
        .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(overlay_area);
    f.render_widget(block, overlay_area);

    let lines = build_hint_lines(hint_state, inner.width as usize);
    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, inner);

    true
}

fn build_hint_lines(hint_state: &HintContext, width: usize) -> Vec<Line<'static>> {
    let items: Vec<(String, Option<String>)> = match hint_state {
        HintContext::Inactive => return vec![],

        HintContext::Tags { matches, .. } => matches
            .iter()
            .take(MAX_ITEMS)
            .map(|t| (format!("#{t}"), None))
            .collect(),

        HintContext::Commands { matches, .. } => matches
            .iter()
            .take(MAX_ITEMS)
            .map(|h| (format!(":{}", h.command), None))
            .collect(),

        HintContext::FilterTypes { matches, .. } => matches
            .iter()
            .take(MAX_ITEMS)
            .map(|h| (h.syntax.to_string(), None))
            .collect(),

        HintContext::DateOps { matches, .. } => matches
            .iter()
            .take(MAX_ITEMS)
            .map(|h| (h.syntax.to_string(), None))
            .collect(),

        HintContext::Negation { matches, .. } => matches
            .iter()
            .take(MAX_ITEMS)
            .map(|h| (h.syntax.to_string(), None))
            .collect(),
    };

    if items.is_empty() {
        return vec![];
    }

    // Calculate columns based on available width
    let num_cols = (width / COLUMN_WIDTH).clamp(1, MAX_COLUMNS);
    let rows = items.len().div_ceil(num_cols);

    let mut lines = Vec::with_capacity(rows);

    for row in 0..rows {
        let mut spans = Vec::new();

        for col in 0..num_cols {
            let idx = col * rows + row;
            if idx < items.len() {
                let (key, _) = &items[idx];
                let display = format!("{:width$}", key, width = COLUMN_WIDTH);
                spans.push(Span::styled(display, Style::default().fg(Color::Cyan)));
            }
        }

        lines.push(Line::from(spans));
    }

    lines
}
