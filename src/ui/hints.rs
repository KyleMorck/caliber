use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use super::theme;

use crate::app::HintContext;

pub const HINT_OVERLAY_HEIGHT: u16 = 5;
const COLUMN_WIDTH: usize = 16;

pub fn render_hint_overlay(f: &mut Frame, hint_state: &HintContext, footer_area: Rect) -> bool {
    if matches!(hint_state, HintContext::Inactive) {
        return false;
    }

    let overlay_area = Rect {
        x: footer_area.x,
        y: footer_area.y.saturating_sub(HINT_OVERLAY_HEIGHT),
        width: footer_area.width,
        height: HINT_OVERLAY_HEIGHT,
    };

    if overlay_area.height == 0 || overlay_area.width < 20 {
        return false;
    }

    f.render_widget(Clear, overlay_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::HINT_BORDER).dim());

    let inner = block.inner(overlay_area);
    f.render_widget(block, overlay_area);

    let lines = build_hint_lines(hint_state, inner.width as usize, inner.height as usize);
    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, inner);

    true
}

fn build_guidance_lines(message: &str, max_rows: usize) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();
    for _ in 0..max_rows.saturating_sub(1) {
        lines.push(Line::from(""));
    }
    lines.push(Line::from(Span::styled(
        message.to_string(),
        Style::default().fg(theme::HINT_GUIDANCE).italic(),
    )));
    lines
}

fn build_hint_lines(hint_state: &HintContext, width: usize, max_rows: usize) -> Vec<Line<'static>> {
    let is_negation = matches!(hint_state, HintContext::Negation { .. });
    let negation_prefix = if is_negation { "not:" } else { "" };

    if let HintContext::GuidanceMessage { message } = hint_state {
        return build_guidance_lines(message, max_rows);
    }
    if let HintContext::Negation { inner } = hint_state
        && let HintContext::GuidanceMessage { message } = inner.as_ref()
    {
        return build_guidance_lines(message, max_rows);
    }

    let description = hint_state.description();
    let hint_color = hint_state.color();
    let items = hint_state.display_items(negation_prefix);

    let num_cols = width / COLUMN_WIDTH;
    if hint_color == Color::Reset || items.is_empty() || max_rows == 0 || num_cols == 0 {
        return vec![];
    }

    let hint_rows = if description.is_some() {
        max_rows.saturating_sub(1)
    } else {
        max_rows
    };

    let mut lines: Vec<Line<'static>> = Vec::new();

    if hint_rows > 0 {
        let mut row_spans: Vec<Vec<Span>> = vec![Vec::new(); hint_rows];

        for (i, item) in items.iter().enumerate() {
            let col = i / hint_rows;
            let row = i % hint_rows;

            if col >= num_cols {
                break;
            }

            let display = format!("{:width$}", item, width = COLUMN_WIDTH);
            row_spans[row].push(Span::styled(display, Style::default().fg(hint_color)));
        }

        for spans in row_spans {
            lines.push(if spans.is_empty() {
                Line::from("")
            } else {
                Line::from(spans)
            });
        }
    }

    if let Some(desc) = description {
        let truncated = if desc.len() > width {
            format!("{}…", &desc[..width.saturating_sub(1)])
        } else {
            desc.to_string()
        };
        lines.push(Line::from(Span::styled(
            truncated,
            Style::default().fg(theme::HINT_DESC),
        )));
    }

    lines
}
