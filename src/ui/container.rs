use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::Line as RatatuiLine,
    widgets::{Block, BorderType, Borders, Paragraph},
};

use super::context::RenderContext;
use super::layout::padded_content_area_with_buffer;
use super::model::ListModel;

pub struct ContainerConfig {
    pub title: Option<RatatuiLine<'static>>,
    pub border_color: Color,
    pub focused_border_color: Option<Color>,
    pub padded: bool,
    pub borders: Borders,
    pub rounded: bool,
    pub bottom_buffer: u16,
}

/// Shared container config for view content panels (no borders, with padding).
#[must_use]
pub fn view_content_container_config(border_color: Color) -> ContainerConfig {
    ContainerConfig {
        title: None,
        border_color,
        focused_border_color: Some(super::theme::BORDER_DEFAULT),
        padded: true,
        borders: Borders::NONE,
        rounded: false,
        bottom_buffer: super::theme::ENTRY_LIST_BOTTOM_BUFFER,
    }
}

pub struct ContainerLayout {
    pub main_area: Rect,
    pub content_area: Rect,
}

#[allow(dead_code)]
pub fn render_container(
    f: &mut Frame<'_>,
    context: &RenderContext,
    config: &ContainerConfig,
    focused: bool,
) -> ContainerLayout {
    render_container_in_area(f, context.main_area, config, focused)
}

pub fn render_container_in_area(
    f: &mut Frame<'_>,
    area: Rect,
    config: &ContainerConfig,
    focused: bool,
) -> ContainerLayout {
    let border_color = if focused {
        config.focused_border_color.unwrap_or(config.border_color)
    } else {
        config.border_color
    };

    let border_type = if config.rounded {
        BorderType::Rounded
    } else {
        BorderType::Plain
    };

    let mut block = Block::default()
        .borders(config.borders)
        .border_type(border_type)
        .border_style(Style::default().fg(border_color));

    if let Some(title) = config.title.clone() {
        block = block.title_top(title);
    }

    f.render_widget(block, area);

    let content_area = content_area_for(area, config);

    ContainerLayout {
        main_area: area,
        content_area,
    }
}

pub fn content_area_for(area: Rect, config: &ContainerConfig) -> Rect {
    let inner = Block::default().borders(config.borders).inner(area);
    if config.padded {
        padded_content_area_with_buffer(inner, config.bottom_buffer)
    } else if config.bottom_buffer > 0 {
        Rect {
            height: inner.height.saturating_sub(config.bottom_buffer),
            ..inner
        }
    } else {
        inner
    }
}

pub fn render_list(f: &mut Frame<'_>, list: &ListModel, layout: &ContainerLayout) {
    let scroll_offset = list.scroll.offset;
    let lines = list.lines();

    #[allow(clippy::cast_possible_truncation)]
    let content = Paragraph::new(lines).scroll((scroll_offset as u16, 0));
    f.render_widget(content, layout.content_area);
}
