use ratatui::text::{Line as RatatuiLine, Span};

#[derive(Clone, Debug)]
pub struct ListModel {
    pub header: Option<RatatuiLine<'static>>,
    pub rows: Vec<RowModel>,
    pub scroll: ScrollModel,
}

impl ListModel {
    #[must_use]
    pub fn from_rows(
        header: Option<RatatuiLine<'static>>,
        rows: Vec<RowModel>,
        offset: usize,
    ) -> Self {
        let total = rows.len() + header.as_ref().map_or(0, |_| 1);
        Self {
            header,
            rows,
            scroll: ScrollModel::new(offset, total, 0),
        }
    }

    #[must_use]
    pub fn to_lines(self) -> Vec<RatatuiLine<'static>> {
        let header_count = self.header.as_ref().map_or(0, |_| 1);
        let mut lines = Vec::with_capacity(self.rows.len() + header_count);
        if let Some(header) = self.header {
            lines.push(header);
        }
        lines.extend(self.rows.into_iter().map(RowModel::to_line));
        lines
    }
}

#[derive(Clone, Debug)]
pub struct RowModel {
    pub indicator: Option<Span<'static>>,
    pub prefix: Option<Span<'static>>,
    pub content: Vec<Span<'static>>,
    pub suffix: Option<Span<'static>>,
}

impl RowModel {
    #[must_use]
    pub fn new(
        indicator: Option<Span<'static>>,
        prefix: Option<Span<'static>>,
        content: Vec<Span<'static>>,
        suffix: Option<Span<'static>>,
    ) -> Self {
        Self {
            indicator,
            prefix,
            content,
            suffix,
        }
    }

    #[must_use]
    pub fn from_spans(spans: Vec<Span<'static>>) -> Self {
        Self {
            indicator: None,
            prefix: None,
            content: spans,
            suffix: None,
        }
    }

    #[must_use]
    pub fn to_line(self) -> RatatuiLine<'static> {
        let mut spans = Vec::with_capacity(
            self.content.len()
                + usize::from(self.indicator.is_some())
                + usize::from(self.prefix.is_some())
                + usize::from(self.suffix.is_some()),
        );
        if let Some(indicator) = self.indicator {
            spans.push(indicator);
        }
        if let Some(prefix) = self.prefix {
            spans.push(prefix);
        }
        spans.extend(self.content);
        if let Some(suffix) = self.suffix {
            spans.push(suffix);
        }
        RatatuiLine::from(spans)
    }
}

#[derive(Clone, Debug)]
pub struct ScrollModel {
    pub offset: usize,
    pub total: usize,
    pub visible: usize,
}

impl ScrollModel {
    #[must_use]
    pub fn new(offset: usize, total: usize, visible: usize) -> Self {
        Self {
            offset,
            total,
            visible,
        }
    }
}
