use ratatui::layout::{Constraint, Direction, Layout, Rect};

#[derive(Clone)]
pub struct RenderContext {
    pub size: Rect,
    pub header_area: Rect,
    pub heading_area: Rect,
    pub main_area: Rect,
    pub footer_area: Rect,
    pub content_area: Rect,
    pub sidebar_area: Option<Rect>,
}

impl RenderContext {
    #[must_use]
    pub fn new(size: Rect) -> Self {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(2),
                Constraint::Min(3),
                Constraint::Length(1),
            ])
            .split(size);
        let header_area = chunks[0];
        let heading_area = chunks[1];
        let main_area = chunks[2];
        let footer_area = chunks[3];

        Self {
            size,
            header_area,
            heading_area,
            main_area,
            footer_area,
            content_area: main_area,
            sidebar_area: None,
        }
    }

    #[must_use]
    pub fn with_sidebar(&self, sidebar_width: u16) -> Self {
        if sidebar_width == 0 {
            return self.clone();
        }

        let content_width = self.main_area.width.saturating_sub(sidebar_width);
        let content_area = Rect {
            x: self.main_area.x,
            y: self.main_area.y,
            width: content_width.max(1),
            height: self.main_area.height,
        };
        let sidebar_area = Rect {
            x: self.main_area.x.saturating_add(content_width),
            y: self.main_area.y,
            width: sidebar_width.min(self.main_area.width),
            height: self.main_area.height,
        };

        Self {
            content_area,
            sidebar_area: Some(sidebar_area),
            ..self.clone()
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
