use chrono::{Datelike, NaiveDate, Weekday};

#[derive(Debug, Clone, PartialEq)]
pub enum EntryType {
    Task { completed: bool },
    Note,
    Event,
}

impl EntryType {
    #[must_use]
    pub fn prefix(&self) -> &'static str {
        match self {
            Self::Task { completed: false } => "- [ ] ",
            Self::Task { completed: true } => "- [x] ",
            Self::Note => "- ",
            Self::Event => "* ",
        }
    }

    #[must_use]
    pub fn cycle(&self) -> Self {
        match self {
            Self::Task { .. } => Self::Note,
            Self::Note => Self::Event,
            Self::Event => Self::Task { completed: false },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub entry_type: EntryType,
    pub content: String,
}

impl Entry {
    #[must_use]
    pub fn new_task(content: &str) -> Self {
        Self {
            entry_type: EntryType::Task { completed: false },
            content: content.to_string(),
        }
    }

    pub fn prefix(&self) -> &'static str {
        self.entry_type.prefix()
    }

    pub fn toggle_complete(&mut self) {
        if let EntryType::Task { completed } = &mut self.entry_type {
            *completed = !*completed;
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Line {
    Entry(Entry),
    Raw(String),
}

/// Filter view entry: wraps Entry with location metadata for operations.
#[derive(Debug, Clone)]
pub struct FilterResult {
    pub source_date: NaiveDate,
    pub line_index: usize,
    pub entry: Entry,
}

/// The kind of projected entry (determines display and toggle behavior).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectedKind {
    /// One-time projection via @MM/DD syntax
    Later,
    /// Repeating projection via @every-* syntax
    Recurring,
}

/// Daily view projected entry: entry appearing on a different date than its source.
#[derive(Debug, Clone)]
pub struct ProjectedEntry {
    pub source_date: NaiveDate,
    pub line_index: usize,
    pub entry: Entry,
    pub kind: ProjectedKind,
}

/// Recurring pattern for @every-* syntax.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecurringPattern {
    /// @every-day - every day
    Daily,
    /// @every-weekday - Monday through Friday
    Weekday,
    /// @every-monday through @every-sunday
    Weekly(Weekday),
    /// @every-1 through @every-31 (day of month)
    Monthly(u8),
}

impl RecurringPattern {
    /// Returns true if this pattern matches the given date.
    #[must_use]
    pub fn matches(&self, date: NaiveDate) -> bool {
        match self {
            Self::Daily => true,
            Self::Weekday => !matches!(date.weekday(), Weekday::Sat | Weekday::Sun),
            Self::Weekly(day) => date.weekday() == *day,
            Self::Monthly(day) => {
                let last_day = last_day_of_month(date);
                if u32::from(*day) > last_day {
                    date.day() == last_day
                } else {
                    date.day() == u32::from(*day)
                }
            }
        }
    }
}

/// Returns the last day of the month for the given date.
fn last_day_of_month(date: NaiveDate) -> u32 {
    let (year, month) = (date.year(), date.month());
    let next_month = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    };
    next_month
        .and_then(|d| d.pred_opt())
        .map(|d| d.day())
        .unwrap_or(28)
}

fn parse_line(line: &str) -> Line {
    let trimmed = line.trim_start();

    if let Some(content) = trimmed.strip_prefix("- [ ] ") {
        return Line::Entry(Entry {
            entry_type: EntryType::Task { completed: false },
            content: content.to_string(),
        });
    }

    if let Some(content) = trimmed.strip_prefix("- [x] ") {
        return Line::Entry(Entry {
            entry_type: EntryType::Task { completed: true },
            content: content.to_string(),
        });
    }

    if let Some(content) = trimmed.strip_prefix("* ") {
        return Line::Entry(Entry {
            entry_type: EntryType::Event,
            content: content.to_string(),
        });
    }

    if let Some(content) = trimmed.strip_prefix("- ") {
        return Line::Entry(Entry {
            entry_type: EntryType::Note,
            content: content.to_string(),
        });
    }

    Line::Raw(line.to_string())
}

#[must_use]
pub fn parse_lines(content: &str) -> Vec<Line> {
    content.lines().map(parse_line).collect()
}

fn serialize_line(line: &Line) -> String {
    match line {
        Line::Entry(entry) => format!("{}{}", entry.prefix(), entry.content),
        Line::Raw(s) => s.clone(),
    }
}

#[must_use]
pub fn serialize_lines(lines: &[Line]) -> String {
    lines
        .iter()
        .map(serialize_line)
        .collect::<Vec<_>>()
        .join("\n")
}
