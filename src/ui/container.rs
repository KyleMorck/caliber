use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::Line as RatatuiLine,
    widgets::{Block, Borders, Paragraph},
};
use unicode_width::UnicodeWidthStr;

use super::context::RenderContext;
use super::model::ListModel;
use super::scroll_indicator::{ScrollIndicatorStyle, scroll_indicator_text};
use super::theme;

pub struct ContainerConfig {
    pub title: Option<RatatuiLine<'static>>,
    pub border_color: Color,
}

impl ContainerConfig {
    #[must_use]
    pub fn daily(title: RatatuiLine<'static>) -> Self {
        Self {
            title: Some(title),
            border_color: theme::BORDER_DAILY,
        }
    }

    #[must_use]
    pub fn filter() -> Self {
        Self {
            title: None,
            border_color: theme::BORDER_FILTER,
        }
    }
}

pub struct ContainerLayout {
    pub main_area: Rect,
    pub content_area: Rect,
}

pub fn render_container(
    f: &mut Frame<'_>,
    context: &RenderContext,
    config: &ContainerConfig,
) -> ContainerLayout {
    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(config.border_color));

    if let Some(title) = config.title.clone() {
        block = block.title_top(title);
    }

    f.render_widget(block, context.main_area);

    ContainerLayout {
        main_area: context.main_area,
        content_area: context.content_area,
    }
}

pub fn render_list(f: &mut Frame<'_>, list: ListModel, layout: &ContainerLayout) {
    let scroll_offset = list.scroll.offset;
    let total_lines = list.scroll.total;
    let lines = list.to_lines();

    #[allow(clippy::cast_possible_truncation)]
    let content = Paragraph::new(lines).scroll((scroll_offset as u16, 0));
    f.render_widget(content, layout.content_area);

    let content_height = layout.content_area.height as usize;
    let can_scroll_up = scroll_offset > 0;
    let can_scroll_down = scroll_offset + content_height < total_lines;

    if let Some(arrows) = scroll_indicator_text(
        can_scroll_up,
        can_scroll_down,
        ScrollIndicatorStyle::Labeled,
    ) {
        let indicator_width = arrows.width() as u16;
        let indicator_area = Rect {
            x: layout
                .main_area
                .x
                .saturating_add(layout.main_area.width.saturating_sub(indicator_width + 1)),
            y: layout
                .main_area
                .y
                .saturating_add(layout.main_area.height.saturating_sub(1)),
            width: indicator_width,
            height: 1,
        };
        let scroll_indicator =
            Paragraph::new(ratatui::text::Span::styled(arrows, Style::default().dim()));
        f.render_widget(scroll_indicator, indicator_area);
    }
}
