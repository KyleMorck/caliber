use std::collections::HashMap;

use chrono::{Days, NaiveDate};
use regex::Regex;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::{LazyLock, RwLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JournalSlot {
    Global,
    Project,
}

struct JournalContext {
    global_path: PathBuf,
    project_path: Option<PathBuf>,
    active: JournalSlot,
}

static JOURNAL_CONTEXT: RwLock<Option<JournalContext>> = RwLock::new(None);

pub fn set_journal_context(global: PathBuf, project: Option<PathBuf>, active: JournalSlot) {
    if let Ok(mut guard) = JOURNAL_CONTEXT.write() {
        *guard = Some(JournalContext {
            global_path: global,
            project_path: project,
            active,
        });
    }
}

#[must_use]
pub fn get_active_slot() -> JournalSlot {
    JOURNAL_CONTEXT
        .read()
        .ok()
        .and_then(|guard| guard.as_ref().map(|ctx| ctx.active))
        .unwrap_or(JournalSlot::Global)
}

pub fn set_active_slot(slot: JournalSlot) {
    if let Ok(mut guard) = JOURNAL_CONTEXT.write()
        && let Some(ctx) = guard.as_mut()
    {
        ctx.active = slot;
    }
}

#[must_use]
pub fn get_project_path() -> Option<PathBuf> {
    JOURNAL_CONTEXT
        .read()
        .ok()
        .and_then(|guard| guard.as_ref().and_then(|ctx| ctx.project_path.clone()))
}

pub fn set_project_path(path: PathBuf) {
    if let Ok(mut guard) = JOURNAL_CONTEXT.write()
        && let Some(ctx) = guard.as_mut()
    {
        ctx.project_path = Some(path);
    }
}

#[must_use]
pub fn get_active_journal_path() -> PathBuf {
    JOURNAL_CONTEXT
        .read()
        .ok()
        .and_then(|guard| {
            guard.as_ref().map(|ctx| match ctx.active {
                JournalSlot::Global => ctx.global_path.clone(),
                JournalSlot::Project => ctx
                    .project_path
                    .clone()
                    .unwrap_or_else(|| ctx.global_path.clone()),
            })
        })
        .unwrap_or_else(get_journal_path)
}

/// Detects if we're in a git repository and returns the project root path.
#[must_use]
pub fn find_git_root() -> Option<PathBuf> {
    let mut current = std::env::current_dir().ok()?;
    loop {
        if current.join(".git").exists() {
            return Some(current);
        }
        if !current.pop() {
            break;
        }
    }
    None
}

/// Detects if a project journal exists and returns its path.
/// Returns Some(path) if .caliber/journal.md exists, None otherwise.
#[must_use]
pub fn detect_project_journal() -> Option<PathBuf> {
    // First check for git root
    if let Some(root) = find_git_root() {
        let project_journal = root.join(".caliber").join("journal.md");
        if project_journal.exists() {
            return Some(project_journal);
        }
        return None;
    }

    // Fallback: check current directory for .caliber/
    let cwd = std::env::current_dir().ok()?;
    let project_journal = cwd.join(".caliber").join("journal.md");
    if project_journal.exists() {
        return Some(project_journal);
    }

    None
}

/// Creates a project journal at .caliber/journal.md in the git root.
pub fn create_project_journal() -> io::Result<PathBuf> {
    let root = find_git_root()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Not in a git repository"))?;

    let caliber_dir = root.join(".caliber");
    fs::create_dir_all(&caliber_dir)?;

    let journal_path = caliber_dir.join("journal.md");
    if !journal_path.exists() {
        fs::write(&journal_path, "")?;
    }

    Ok(journal_path)
}

/// Adds .caliber/ to .gitignore if not already present.
pub fn add_caliber_to_gitignore() -> io::Result<()> {
    let root = find_git_root()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Not in a git repository"))?;

    let gitignore_path = root.join(".gitignore");
    let entry = ".caliber/";

    if gitignore_path.exists() {
        let content = fs::read_to_string(&gitignore_path)?;
        if content.lines().any(|line| line.trim() == entry) {
            return Ok(());
        }
        let mut new_content = content;
        if !new_content.ends_with('\n') && !new_content.is_empty() {
            new_content.push('\n');
        }
        new_content.push_str(entry);
        new_content.push('\n');
        fs::write(&gitignore_path, new_content)?;
    } else {
        fs::write(&gitignore_path, format!("{entry}\n"))?;
    }

    Ok(())
}

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

pub fn load_day_lines(date: NaiveDate) -> io::Result<Vec<Line>> {
    let content = load_day(date)?;
    Ok(parse_lines(&content))
}

pub fn save_day_lines(date: NaiveDate, lines: &[Line]) -> io::Result<()> {
    let content = serialize_lines(lines);
    save_day(date, &content)
}

/// Updates an entry's content at a specific line index for a given date.
/// Returns Ok(true) if update succeeded, Ok(false) if no entry at that index.
pub fn update_entry_content(
    date: NaiveDate,
    line_index: usize,
    content: String,
) -> io::Result<bool> {
    let mut lines = load_day_lines(date)?;
    let updated = if let Some(Line::Entry(entry)) = lines.get_mut(line_index) {
        entry.content = content;
        true
    } else {
        false
    };
    if updated {
        save_day_lines(date, &lines)?;
    }
    Ok(updated)
}

/// Toggles the completion status of a task at a specific line index.
pub fn toggle_entry_complete(date: NaiveDate, line_index: usize) -> io::Result<()> {
    let mut lines = load_day_lines(date)?;
    if let Some(Line::Entry(entry)) = lines.get_mut(line_index) {
        entry.toggle_complete();
    }
    save_day_lines(date, &lines)
}

/// Cycles the entry type (Task -> Note -> Event -> Task) at a specific line index.
/// Returns the new entry type if successful.
pub fn cycle_entry_type(date: NaiveDate, line_index: usize) -> io::Result<Option<EntryType>> {
    let mut lines = load_day_lines(date)?;
    let new_type = if let Some(Line::Entry(entry)) = lines.get_mut(line_index) {
        entry.entry_type = entry.entry_type.cycle();
        Some(entry.entry_type.clone())
    } else {
        None
    };
    save_day_lines(date, &lines)?;
    Ok(new_type)
}

/// Deletes an entry at a specific line index for a given date.
pub fn delete_entry(date: NaiveDate, line_index: usize) -> io::Result<()> {
    let mut lines = load_day_lines(date)?;
    if line_index < lines.len() {
        lines.remove(line_index);
    }
    save_day_lines(date, &lines)
}

pub fn get_journal_path() -> PathBuf {
    crate::config::get_default_journal_path()
}

fn day_header(date: NaiveDate) -> String {
    format!("# {}", date.format("%Y/%m/%d"))
}

pub fn load_journal() -> io::Result<String> {
    let path = get_active_journal_path();
    if path.exists() {
        fs::read_to_string(path)
    } else {
        Ok(String::new())
    }
}

pub fn save_journal(content: &str) -> io::Result<()> {
    let path = get_active_journal_path();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, content)
}

pub fn extract_day_content(journal: &str, date: NaiveDate) -> String {
    let header = day_header(date);

    let Some(start_idx) = journal.find(&header) else {
        return String::new();
    };

    let content_start = start_idx + header.len();
    let after_header = &journal[content_start..];
    let after_header = after_header.strip_prefix('\n').unwrap_or(after_header);
    let end_idx = find_next_day_header(after_header);

    match end_idx {
        Some(idx) => after_header[..idx].trim_end().to_string(),
        None => after_header.trim_end().to_string(),
    }
}

fn parse_day_header(line: &str) -> Option<NaiveDate> {
    if !line.starts_with("# ") {
        return None;
    }
    let rest = &line[2..];
    if rest.len() < 10 {
        return None;
    }
    NaiveDate::parse_from_str(&rest[..10], "%Y/%m/%d").ok()
}

fn is_day_header(line: &str) -> bool {
    parse_day_header(line).is_some()
}

fn find_next_day_header(content: &str) -> Option<usize> {
    let mut byte_pos = 0;
    let mut is_first_line = true;

    for line in content.lines() {
        // Skip first line - this function is called on content after a header,
        // so line 0 is day content, not a potential next header.
        if !is_first_line && is_day_header(line) {
            return Some(byte_pos);
        }
        is_first_line = false;

        byte_pos += line.len();
        if byte_pos < content.len() {
            let next_char = content[byte_pos..].chars().next();
            if next_char == Some('\r') {
                byte_pos += 1;
            }
            if byte_pos < content.len() && content[byte_pos..].starts_with('\n') {
                byte_pos += 1;
            }
        }
    }
    None
}

pub fn update_day_content(journal: &str, date: NaiveDate, new_content: &str) -> String {
    let header = day_header(date);
    let content_is_empty = new_content.trim().is_empty();

    if let Some(start_idx) = journal.find(&header) {
        let (before, after) = split_around_day(journal, start_idx, &header);
        if content_is_empty {
            remove_day(before, after)
        } else {
            replace_day(before, &header, new_content, after)
        }
    } else if content_is_empty {
        journal.to_string()
    } else {
        insert_new_day(journal, date, &header, new_content)
    }
}

fn split_around_day<'a>(journal: &'a str, start_idx: usize, header: &str) -> (&'a str, &'a str) {
    let before = &journal[..start_idx];
    let content_start = start_idx + header.len();
    let after_header = &journal[content_start..];
    let after_header = after_header.strip_prefix('\n').unwrap_or(after_header);

    let after = match find_next_day_header(after_header) {
        Some(idx) => &after_header[idx..],
        None => "",
    };

    (before, after)
}

fn remove_day(before: &str, after: &str) -> String {
    let mut result = before.trim_end().to_string();
    if !result.is_empty() && !after.is_empty() {
        result.push_str("\n\n");
    }
    result.push_str(after.trim_start());
    if result.is_empty() {
        result
    } else {
        result.trim_end().to_string() + "\n"
    }
}

fn replace_day(before: &str, header: &str, content: &str, after: &str) -> String {
    format!("{}{}\n{}\n\n{}", before, header, content.trim_end(), after)
        .trim_end()
        .to_string()
        + "\n"
}

fn insert_new_day(journal: &str, date: NaiveDate, header: &str, content: &str) -> String {
    let new_day = format!("{}\n{}\n", header, content.trim_end());

    let insert_pos = find_insertion_point(journal, date);

    if let Some(pos) = insert_pos {
        let before = journal[..pos].trim_end();
        let after = &journal[pos..];
        if before.is_empty() {
            format!("{}\n{}", new_day.trim_end(), after.trim_start())
                .trim_end()
                .to_string()
                + "\n"
        } else {
            format!(
                "{}\n\n{}\n{}",
                before,
                new_day.trim_end(),
                after.trim_start()
            )
            .trim_end()
            .to_string()
                + "\n"
        }
    } else {
        let mut result = journal.trim_end().to_string();
        if !result.is_empty() {
            result.push_str("\n\n");
        }
        result.push_str(new_day.trim_end());
        result.push('\n');
        result
    }
}

fn find_insertion_point(journal: &str, date: NaiveDate) -> Option<usize> {
    for line in journal.lines() {
        if let Some(existing_date) = parse_day_header(line)
            && existing_date > date
        {
            return journal.find(line);
        }
    }
    None
}

pub fn load_day(date: NaiveDate) -> io::Result<String> {
    let journal = load_journal()?;
    Ok(extract_day_content(&journal, date))
}

pub fn save_day(date: NaiveDate, content: &str) -> io::Result<()> {
    let journal = load_journal()?;
    let updated = update_day_content(&journal, date, content);
    save_journal(&updated)
}

#[derive(Debug, Clone)]
pub struct FilterEntry {
    pub source_date: NaiveDate,
    pub content: String,
    pub line_index: usize,
    pub entry_type: EntryType,
    pub completed: bool,
}

/// An entry from another day that should appear on a target date via @date syntax.
#[derive(Debug, Clone)]
pub struct LaterEntry {
    pub source_date: NaiveDate,
    pub line_index: usize,
    pub content: String,
    pub entry_type: EntryType,
    pub completed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FilterType {
    Task,
    Note,
    Event,
}

#[derive(Debug, Clone, Default)]
pub struct Filter {
    pub entry_type: Option<FilterType>,
    pub completed: Option<bool>,
    pub tags: Vec<String>,
    pub exclude_tags: Vec<String>,
    pub search_terms: Vec<String>,
    pub exclude_terms: Vec<String>,
    pub exclude_types: Vec<FilterType>,
    pub before_date: Option<NaiveDate>,
    pub after_date: Option<NaiveDate>,
    pub overdue: bool,
    pub invalid_tokens: Vec<String>,
}

pub static TAG_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"#([a-zA-Z][a-zA-Z0-9_-]*)").unwrap());

/// Matches @date patterns:
/// - @MM/DD (e.g., @1/9, @01/09)
/// - @MM/DD/YY (e.g., @1/9/26, @01/09/26)
/// - @MM/DD/YYYY (e.g., @1/9/2026, @01/09/2026)
/// - @YYYY/MM/DD (ISO format, e.g., @2026/1/9)
pub static LATER_DATE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"@(\d{4}/\d{1,2}/\d{1,2}|\d{1,2}/\d{1,2}(?:/\d{2,4})?)").unwrap());

/// Matches natural date patterns: @tomorrow, @yesterday, @next-monday, @last-monday, @3d, @-3d
pub static NATURAL_DATE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)@(tomorrow|yesterday|(?:next|last)-(?:mon(?:day)?|tue(?:sday)?|wed(?:nesday)?|thu(?:rsday)?|fri(?:day)?|sat(?:urday)?|sun(?:day)?)|-?[1-9]\d*d)").unwrap()
});

/// Matches favorite tag shortcuts: #1 through #9 and #0
pub static FAVORITE_TAG_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"#([0-9])\b").unwrap());

/// Matches saved filter shortcuts: $name (alphanumeric + underscore)
pub static SAVED_FILTER_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\$(\w+)\b").unwrap());

#[must_use]
pub fn extract_tags(content: &str) -> Vec<String> {
    TAG_REGEX
        .captures_iter(content)
        .map(|cap| cap[1].to_string())
        .collect()
}

/// Parses a date string (without @) into a NaiveDate.
/// Tries ISO (YYYY/MM/DD), MM/DD/YYYY, MM/DD/YY, and MM/DD formats.
/// For MM/DD without year, uses "always future" logic: if date has passed
/// this year, assumes next year.
#[must_use]
pub fn parse_later_date(date_str: &str, today: NaiveDate) -> Option<NaiveDate> {
    use chrono::Datelike;

    // YYYY/MM/DD (only if first part is exactly 4 digits)
    if let Some(first_slash) = date_str.find('/')
        && first_slash == 4
        && date_str[..4].chars().all(|c| c.is_ascii_digit())
        && let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y/%m/%d")
    {
        return Some(date);
    }

    // MM/DD/YYYY or MM/DD/YY
    if date_str.matches('/').count() == 2 {
        let parts: Vec<&str> = date_str.split('/').collect();
        if parts.len() == 3
            && let (Ok(month), Ok(day), Ok(year)) = (
                parts[0].parse::<u32>(),
                parts[1].parse::<u32>(),
                parts[2].parse::<i32>(),
            )
        {
            // If year is 2 digits, assume 20xx
            let full_year = if year < 100 { 2000 + year } else { year };
            if let Some(date) = NaiveDate::from_ymd_opt(full_year, month, day) {
                return Some(date);
            }
        }
    }

    // MM/DD (no year) - use "always future" logic
    let parts: Vec<&str> = date_str.split('/').collect();
    if parts.len() == 2 {
        let month: u32 = parts[0].parse().ok()?;
        let day: u32 = parts[1].parse().ok()?;
        if let Some(date) = NaiveDate::from_ymd_opt(today.year(), month, day) {
            if date < today {
                return NaiveDate::from_ymd_opt(today.year() + 1, month, day);
            }
            return Some(date);
        }
    }

    None
}

/// Parses a weekday name (full or abbreviated) into chrono::Weekday.
fn parse_weekday(s: &str) -> Option<chrono::Weekday> {
    use chrono::Weekday;
    match s.to_lowercase().as_str() {
        "monday" | "mon" => Some(Weekday::Mon),
        "tuesday" | "tue" => Some(Weekday::Tue),
        "wednesday" | "wed" => Some(Weekday::Wed),
        "thursday" | "thu" => Some(Weekday::Thu),
        "friday" | "fri" => Some(Weekday::Fri),
        "saturday" | "sat" => Some(Weekday::Sat),
        "sunday" | "sun" => Some(Weekday::Sun),
        _ => None,
    }
}

/// Returns the next occurrence of a weekday after today (never returns today).
fn next_weekday_from(today: NaiveDate, target: chrono::Weekday) -> Option<NaiveDate> {
    use chrono::Datelike;
    let today_wd = today.weekday().num_days_from_monday();
    let target_wd = target.num_days_from_monday();

    let days_ahead = if target_wd > today_wd {
        target_wd - today_wd
    } else {
        7 - today_wd + target_wd
    };
    let days_ahead = if days_ahead == 0 { 7 } else { days_ahead };

    today.checked_add_days(Days::new(u64::from(days_ahead)))
}

/// Returns the most recent occurrence of a weekday before today (never returns today).
fn prev_weekday_from(today: NaiveDate, target: chrono::Weekday) -> Option<NaiveDate> {
    use chrono::Datelike;
    let today_wd = today.weekday().num_days_from_monday();
    let target_wd = target.num_days_from_monday();

    let days_back = if target_wd < today_wd {
        today_wd - target_wd
    } else {
        7 - target_wd + today_wd
    };
    let days_back = if days_back == 0 { 7 } else { days_back };

    today.checked_sub_days(Days::new(u64::from(days_back)))
}

/// Parses natural language date expressions: tomorrow, yesterday, next-monday, last-monday, 3d, -3d.
/// Falls back to parse_later_date for standard formats.
#[must_use]
pub fn parse_natural_date(input: &str, today: NaiveDate) -> Option<NaiveDate> {
    let input_lower = input.to_lowercase();

    if input_lower == "tomorrow" {
        return today.checked_add_days(Days::new(1));
    }

    if input_lower == "yesterday" {
        return today.checked_sub_days(Days::new(1));
    }

    // Handle Xd (future) and -Xd (past) patterns
    if let Some(days_str) = input_lower.strip_suffix('d') {
        if let Some(neg_days_str) = days_str.strip_prefix('-') {
            if let Ok(days) = neg_days_str.parse::<u64>()
                && days > 0
            {
                return today.checked_sub_days(Days::new(days));
            }
        } else if let Ok(days) = days_str.parse::<u64>()
            && days > 0
        {
            return today.checked_add_days(Days::new(days));
        }
    }

    if let Some(weekday_str) = input_lower.strip_prefix("next-")
        && let Some(target_weekday) = parse_weekday(weekday_str)
    {
        return next_weekday_from(today, target_weekday);
    }

    if let Some(weekday_str) = input_lower.strip_prefix("last-")
        && let Some(target_weekday) = parse_weekday(weekday_str)
    {
        return prev_weekday_from(today, target_weekday);
    }

    parse_later_date(input, today)
}

/// Replaces natural date patterns (@tomorrow, @yesterday, @next-mon, @last-mon, @3d, @-3d) with @MM/DD format.
#[must_use]
pub fn normalize_natural_dates(content: &str, today: NaiveDate) -> String {
    let mut result = content.to_string();

    for cap in NATURAL_DATE_REGEX.captures_iter(content) {
        if let Some(m) = cap.get(0) {
            let natural_str = &cap[1];
            if let Some(date) = parse_natural_date(natural_str, today) {
                let normalized = format!("@{}/{}", date.format("%m"), date.format("%d"));
                result = result.replacen(m.as_str(), &normalized, 1);
            }
        }
    }

    result
}

/// Replaces favorite tag shortcuts (#0 through #9) with actual tags from config.
/// Tags that don't exist in config are left unchanged.
#[must_use]
pub fn expand_favorite_tags(content: &str, favorite_tags: &HashMap<String, String>) -> String {
    let mut result = content.to_string();

    for cap in FAVORITE_TAG_REGEX.captures_iter(content) {
        if let Some(m) = cap.get(0) {
            let digit = &cap[1];
            if let Some(tag) = favorite_tags.get(digit).filter(|s| !s.is_empty()) {
                result = result.replacen(m.as_str(), &format!("#{tag}"), 1);
            }
        }
    }

    result
}

/// Expands saved filter shortcuts ($name) with their definitions from config.
/// Returns the expanded query and a list of unknown filter names.
#[must_use]
pub fn expand_saved_filters(
    query: &str,
    filters: &HashMap<String, String>,
) -> (String, Vec<String>) {
    let mut result = query.to_string();
    let mut unknown = Vec::new();

    for cap in SAVED_FILTER_REGEX.captures_iter(query) {
        if let Some(m) = cap.get(0) {
            let name = &cap[1];
            if let Some(expansion) = filters.get(name) {
                result = result.replacen(m.as_str(), expansion, 1);
            } else {
                unknown.push(m.as_str().to_string());
            }
        }
    }

    (result, unknown)
}

/// Extracts the target date from entry content if it contains an @date pattern.
#[must_use]
pub fn extract_target_date(content: &str, today: NaiveDate) -> Option<NaiveDate> {
    LATER_DATE_REGEX
        .captures(content)
        .and_then(|cap| cap.get(1))
        .and_then(|m| parse_later_date(m.as_str(), today))
}

/// Like parse_later_date but for MM/DD format prefers the most recent past occurrence.
/// Used for overdue checking where we want to interpret @12/30 on 1/1 as last year.
#[must_use]
fn parse_date_prefer_past(date_str: &str, today: NaiveDate) -> Option<NaiveDate> {
    use chrono::Datelike;

    // For formats with explicit year (MM/DD/YY, MM/DD/YYYY, or YYYY/MM/DD), parse normally
    if date_str.matches('/').count() == 2 {
        return parse_later_date(date_str, today);
    }

    // MM/DD - prefer past (most recent occurrence)
    let parts: Vec<&str> = date_str.split('/').collect();
    if parts.len() == 2 {
        let month: u32 = parts[0].parse().ok()?;
        let day: u32 = parts[1].parse().ok()?;

        // Try current year first
        if let Some(date) = NaiveDate::from_ymd_opt(today.year(), month, day) {
            if date <= today {
                return Some(date);
            }
            // Date is in future this year, so use last year
            return NaiveDate::from_ymd_opt(today.year() - 1, month, day);
        }
    }

    None
}

/// Extracts target date preferring past interpretation (for overdue checking).
#[must_use]
fn extract_target_date_prefer_past(content: &str, today: NaiveDate) -> Option<NaiveDate> {
    LATER_DATE_REGEX
        .captures(content)
        .and_then(|cap| cap.get(1))
        .and_then(|m| parse_date_prefer_past(m.as_str(), today))
}

/// Collects all entries with @date matching the target date.
/// Entries from the target date itself are excluded (they're regular entries).
pub fn collect_later_entries_for_date(target_date: NaiveDate) -> io::Result<Vec<LaterEntry>> {
    let journal = load_journal()?;
    let mut entries = Vec::new();
    let mut current_date: Option<NaiveDate> = None;
    let mut line_index_in_day: usize = 0;

    for line in journal.lines() {
        if let Some(date) = parse_day_header(line) {
            current_date = Some(date);
            line_index_in_day = 0;
            continue;
        }

        if let Some(source_date) = current_date {
            // Skip entries from the target day itself (they're regular entries)
            if source_date == target_date {
                line_index_in_day += 1;
                continue;
            }

            let parsed = parse_line(line);
            if let Line::Entry(entry) = parsed
                && let Some(entry_target) = extract_target_date(&entry.content, target_date)
                && entry_target == target_date
            {
                let completed = matches!(entry.entry_type, EntryType::Task { completed: true });
                entries.push(LaterEntry {
                    source_date,
                    line_index: line_index_in_day,
                    content: entry.content,
                    entry_type: entry.entry_type,
                    completed,
                });
            }
            line_index_in_day += 1;
        }
    }

    // Sort by source date (chronologically - older first)
    entries.sort_by_key(|entry| entry.source_date);
    Ok(entries)
}

fn parse_type_keyword(s: &str) -> Option<FilterType> {
    match s {
        "tasks" | "task" | "t" => Some(FilterType::Task),
        "notes" | "note" | "n" => Some(FilterType::Note),
        "events" | "event" | "e" => Some(FilterType::Event),
        _ => None,
    }
}

#[must_use]
pub fn parse_filter_query(query: &str) -> Filter {
    let mut filter = Filter::default();
    let today = chrono::Local::now().date_naive();

    for token in query.split_whitespace() {
        // Date filters: @before:DATE, @after:DATE, @overdue
        if let Some(date_str) = token.strip_prefix("@before:") {
            if filter.before_date.is_some() {
                filter
                    .invalid_tokens
                    .push("Multiple @before dates".to_string());
            } else if let Some(date) = parse_natural_date(date_str, today) {
                filter.before_date = Some(date);
            } else {
                filter.invalid_tokens.push(token.to_string());
            }
            continue;
        }
        if let Some(date_str) = token.strip_prefix("@after:") {
            if filter.after_date.is_some() {
                filter
                    .invalid_tokens
                    .push("Multiple @after dates".to_string());
            } else if let Some(date) = parse_natural_date(date_str, today) {
                filter.after_date = Some(date);
            } else {
                filter.invalid_tokens.push(token.to_string());
            }
            continue;
        }
        if token == "@overdue" {
            filter.overdue = true;
            continue;
        }
        // Any other @command is invalid
        if token.starts_with('@') && token.contains(':') {
            filter.invalid_tokens.push(token.to_string());
            continue;
        }

        if let Some(negated) = token.strip_prefix("not:") {
            if let Some(tag) = negated.strip_prefix('#') {
                filter.exclude_tags.push(tag.to_string());
            } else if let Some(type_str) = negated.strip_prefix('!') {
                if let Some(filter_type) = parse_type_keyword(type_str) {
                    filter.exclude_types.push(filter_type);
                } else {
                    filter.invalid_tokens.push(token.to_string());
                }
            } else if !negated.is_empty() {
                filter.exclude_terms.push(negated.to_string());
            }
        } else if let Some(type_str) = token.strip_prefix('!') {
            let (base_type, modifier) = if let Some(idx) = type_str.find('/') {
                (&type_str[..idx], Some(&type_str[idx + 1..]))
            } else {
                (type_str, None)
            };

            let new_type = match base_type {
                "tasks" | "task" | "t" => Some(FilterType::Task),
                "notes" | "note" | "n" => Some(FilterType::Note),
                "events" | "event" | "e" => Some(FilterType::Event),
                _ => None,
            };

            if let Some(new_type) = new_type {
                if filter.entry_type.is_some() && filter.entry_type != Some(new_type.clone()) {
                    filter
                        .invalid_tokens
                        .push("Multiple entry types".to_string());
                } else {
                    filter.entry_type = Some(new_type);
                    if base_type == "tasks" || base_type == "task" || base_type == "t" {
                        filter.completed = match modifier {
                            Some("done" | "completed") => Some(true),
                            Some("all") => None,
                            _ => Some(false),
                        };
                    }
                }
            } else {
                filter.invalid_tokens.push(token.to_string());
            }
        } else if let Some(tag) = token.strip_prefix('#') {
            filter.tags.push(tag.to_string());
        } else if !token.is_empty() {
            filter.search_terms.push(token.to_string());
        }
    }

    filter
}

pub fn collect_filtered_entries(filter: &Filter) -> io::Result<Vec<FilterEntry>> {
    if !filter.invalid_tokens.is_empty() {
        return Ok(Vec::new());
    }

    let journal = load_journal()?;
    let mut entries = Vec::new();
    let mut current_date: Option<NaiveDate> = None;
    let mut line_index_in_day: usize = 0;
    let today = chrono::Local::now().date_naive();

    for line in journal.lines() {
        if let Some(date) = parse_day_header(line) {
            current_date = Some(date);
            line_index_in_day = 0;
            continue;
        }

        if let Some(source_date) = current_date {
            // Date filters on day header
            if let Some(before) = filter.before_date
                && source_date > before
            {
                line_index_in_day += 1;
                continue;
            }
            if let Some(after) = filter.after_date
                && source_date < after
            {
                line_index_in_day += 1;
                continue;
            }

            let parsed = parse_line(line);
            if let Line::Entry(entry) = parsed {
                // Overdue filter: entry must have @date targeting before today
                if filter.overdue {
                    let target = extract_target_date_prefer_past(&entry.content, today);
                    if target.is_none() || target.unwrap() >= today {
                        line_index_in_day += 1;
                        continue;
                    }
                }

                if entry_matches_filter(&entry, filter) {
                    let completed = matches!(entry.entry_type, EntryType::Task { completed: true });
                    entries.push(FilterEntry {
                        source_date,
                        content: entry.content,
                        line_index: line_index_in_day,
                        entry_type: entry.entry_type,
                        completed,
                    });
                }
            }
            line_index_in_day += 1;
        }
    }

    entries.sort_by_key(|entry| entry.source_date);
    Ok(entries)
}

fn entry_type_to_filter_type(entry_type: &EntryType) -> FilterType {
    match entry_type {
        EntryType::Task { .. } => FilterType::Task,
        EntryType::Note => FilterType::Note,
        EntryType::Event => FilterType::Event,
    }
}

fn entry_matches_filter(entry: &Entry, filter: &Filter) -> bool {
    let entry_filter_type = entry_type_to_filter_type(&entry.entry_type);

    if let Some(ref filter_type) = filter.entry_type
        && &entry_filter_type != filter_type
    {
        return false;
    }

    for excluded_type in &filter.exclude_types {
        if &entry_filter_type == excluded_type {
            return false;
        }
    }

    if let Some(want_completed) = filter.completed
        && let EntryType::Task { completed } = entry.entry_type
        && completed != want_completed
    {
        return false;
    }

    let entry_tags = extract_tags(&entry.content);

    for required_tag in &filter.tags {
        if !entry_tags
            .iter()
            .any(|t| t.eq_ignore_ascii_case(required_tag))
        {
            return false;
        }
    }

    for excluded_tag in &filter.exclude_tags {
        if entry_tags
            .iter()
            .any(|t| t.eq_ignore_ascii_case(excluded_tag))
        {
            return false;
        }
    }

    let content_lower = entry.content.to_lowercase();

    for term in &filter.search_terms {
        if !content_lower.contains(&term.to_lowercase()) {
            return false;
        }
    }

    for term in &filter.exclude_terms {
        if content_lower.contains(&term.to_lowercase()) {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_round_trip_parsing() {
        let original = "- [ ] Task one\n- [x] Task done\n- A note\n* An event\nRaw line";
        let lines = parse_lines(original);
        let serialized = serialize_lines(&lines);
        assert_eq!(serialized, original);
    }

    #[test]
    fn test_round_trip_with_blank_lines() {
        let original = "- [ ] Task\n\n- Note after blank";
        let lines = parse_lines(original);
        let serialized = serialize_lines(&lines);
        assert_eq!(serialized, original);
    }

    #[test]
    fn test_extract_day_content_multiple_days() {
        let journal = "# 2024/01/15\n- Task 1\n\n# 2024/01/16\n- Task 2\n";

        let date1 = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        assert_eq!(extract_day_content(journal, date1), "- Task 1");

        let date2 = NaiveDate::from_ymd_opt(2024, 1, 16).unwrap();
        assert_eq!(extract_day_content(journal, date2), "- Task 2");
    }

    #[test]
    fn test_update_day_content_preserves_other_days() {
        let journal =
            "# 2024/01/14\n- Day 14\n\n# 2024/01/15\n- Old task\n\n# 2024/01/16\n- Day 16\n";
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let updated = update_day_content(journal, date, "- Updated task");

        assert!(updated.contains("# 2024/01/14\n- Day 14"));
        assert!(updated.contains("# 2024/01/15\n- Updated task"));
        assert!(updated.contains("# 2024/01/16\n- Day 16"));
    }

    #[test]
    fn test_parse_natural_date_all_formats() {
        let today = NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(); // Monday

        // tomorrow/yesterday
        assert_eq!(
            parse_natural_date("tomorrow", today),
            NaiveDate::from_ymd_opt(2026, 1, 6)
        );
        assert_eq!(
            parse_natural_date("yesterday", today),
            NaiveDate::from_ymd_opt(2026, 1, 4)
        );

        // relative days
        assert_eq!(
            parse_natural_date("3d", today),
            NaiveDate::from_ymd_opt(2026, 1, 8)
        );
        assert_eq!(
            parse_natural_date("-3d", today),
            NaiveDate::from_ymd_opt(2026, 1, 2)
        );

        // weekdays
        assert_eq!(
            parse_natural_date("next-monday", today),
            NaiveDate::from_ymd_opt(2026, 1, 12)
        );
        assert_eq!(
            parse_natural_date("last-friday", today),
            NaiveDate::from_ymd_opt(2026, 1, 2)
        );

        // fallback to standard format
        assert_eq!(
            parse_natural_date("1/15", today),
            NaiveDate::from_ymd_opt(2026, 1, 15)
        );
    }

    #[test]
    fn test_normalize_natural_dates() {
        let today = NaiveDate::from_ymd_opt(2026, 1, 5).unwrap();

        assert_eq!(
            normalize_natural_dates("Call dentist @tomorrow", today),
            "Call dentist @01/06"
        );
        assert_eq!(
            normalize_natural_dates("Review @3d and @-3d", today),
            "Review @01/08 and @01/02"
        );
        assert_eq!(
            normalize_natural_dates("Meeting @next-monday", today),
            "Meeting @01/12"
        );
    }

    #[test]
    fn test_filter_combined() {
        let filter = parse_filter_query("!tasks #work @after:1/1 @before:1/31");
        assert_eq!(filter.entry_type, Some(FilterType::Task));
        assert_eq!(filter.tags, vec!["work"]);
        assert!(filter.after_date.is_some());
        assert!(filter.before_date.is_some());
        assert!(filter.invalid_tokens.is_empty());
    }

    #[test]
    fn test_filter_invalid_tokens() {
        assert!(!parse_filter_query("!tas").invalid_tokens.is_empty());
        assert!(!parse_filter_query("!tasks !notes").invalid_tokens.is_empty());
        assert!(!parse_filter_query("@before:1/1 @before:1/15")
            .invalid_tokens
            .is_empty());
    }
}
