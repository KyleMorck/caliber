use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders},
};

pub struct RenderContext {
    pub size: Rect,
    pub header_area: Rect,
    pub main_area: Rect,
    pub footer_area: Rect,
    pub content_area: Rect,
    pub content_width: usize,
    pub scroll_height: usize,
}

impl RenderContext {
    #[must_use]
    pub fn new(size: Rect) -> Self {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(3),
                Constraint::Length(1),
            ])
            .split(size);
        let header_area = chunks[0];
        let main_area = chunks[1];
        let footer_area = chunks[2];

        let inner = Block::default().borders(Borders::ALL).inner(main_area);
        let content_area = super::layout::padded_content_area(inner);

        let content_height = content_area.height as usize;
        let scroll_height = content_height;
        let content_width = content_area.width as usize;

        Self {
            size,
            header_area,
            main_area,
            footer_area,
            content_area,
            content_width,
            scroll_height,
        }
    }

    #[must_use]
    pub fn for_test(width: u16, height: u16) -> Self {
        Self::new(Rect {
            x: 0,
            y: 0,
            width,
            height,
        })
    }
}
