use std::io;

use chrono::{Datelike, Days, Local, NaiveDate};

use crate::config::{Config, resolve_path};
use crate::cursor::CursorBuffer;
use crate::storage::{self, Entry, EntryType, FilterEntry, JournalSlot, LaterEntry, Line};

pub const DAILY_HEADER_LINES: usize = 1;
pub const FILTER_HEADER_LINES: usize = 1;
pub const DATE_SUFFIX_WIDTH: usize = " (MM/DD)".len();

/// State specific to the Daily view
#[derive(Clone)]
pub struct DailyState {
    pub selected: usize,
    pub scroll_offset: usize,
    pub original_lines: Option<Vec<Line>>,
    pub later_entries: Vec<LaterEntry>,
}

impl DailyState {
    #[must_use]
    pub fn new(entry_count: usize, later_entries: Vec<LaterEntry>) -> Self {
        // Default selection: if there are later entries, start at first later entry (index 0)
        // Otherwise start at last regular entry
        let selected = if later_entries.is_empty() {
            entry_count.saturating_sub(1)
        } else {
            0
        };

        Self {
            selected,
            scroll_offset: 0,
            original_lines: None,
            later_entries,
        }
    }

}

/// State specific to the Filter view
#[derive(Clone)]
pub struct FilterState {
    pub query: String,
    pub query_buffer: CursorBuffer,
    pub entries: Vec<FilterEntry>,
    pub selected: usize,
    pub scroll_offset: usize,
}

/// Which view is currently active and its state
#[derive(Clone)]
pub enum ViewMode {
    Daily(DailyState),
    Filter(FilterState),
}

/// Context for what is being edited
#[derive(Clone, Debug, PartialEq)]
pub enum EditContext {
    /// Editing an entry in Daily view
    Daily { entry_index: usize },
    /// Editing an existing entry from Filter view
    FilterEdit {
        date: NaiveDate,
        line_index: usize,
        filter_index: usize,
    },
    /// Quick-adding a new entry from Filter view
    FilterQuickAdd {
        date: NaiveDate,
        entry_type: EntryType,
    },
    /// Editing a later entry from Daily view
    LaterEdit {
        source_date: NaiveDate,
        line_index: usize,
        later_index: usize,
    },
}

/// Context for confirmation dialogs
#[derive(Clone, Debug, PartialEq)]
pub enum ConfirmContext {
    CreateProjectJournal,
    AddToGitignore,
}

/// What keyboard handler to use
#[derive(Clone, Debug, PartialEq)]
pub enum InputMode {
    Normal,
    Edit(EditContext),
    Command,
    Reorder,
    QueryInput,
    Confirm(ConfirmContext),
}

/// Where to insert a new entry
pub enum InsertPosition {
    Bottom,
    Below,
    Above,
}

/// The currently selected item, accounting for hidden completed entries
pub enum SelectedItem<'a> {
    Later {
        index: usize,
        entry: &'a LaterEntry,
    },
    Daily {
        index: usize,
        line_idx: usize,
        entry: &'a Entry,
    },
    Filter {
        index: usize,
        entry: &'a FilterEntry,
    },
    None,
}

/// Internal helper for delete operations (owns data extracted from SelectedItem)
enum DeleteTarget {
    Later {
        source_date: NaiveDate,
        line_index: usize,
        entry_type: EntryType,
        content: String,
    },
    Daily {
        line_idx: usize,
        entry: Entry,
    },
    Filter {
        index: usize,
        source_date: NaiveDate,
        line_index: usize,
        entry_type: EntryType,
        content: String,
    },
}

/// Internal helper for toggle operations
enum ToggleTarget {
    Later {
        source_date: NaiveDate,
        line_index: usize,
    },
    Daily {
        line_idx: usize,
    },
    Filter {
        index: usize,
        source_date: NaiveDate,
        line_index: usize,
    },
}

pub struct App {
    pub current_date: NaiveDate,
    pub lines: Vec<Line>,
    pub entry_indices: Vec<usize>,
    pub view: ViewMode,
    pub input_mode: InputMode,
    pub edit_buffer: Option<CursorBuffer>,
    pub command_buffer: CursorBuffer,
    pub should_quit: bool,
    pub status_message: Option<String>,
    pub show_help: bool,
    pub help_scroll: usize,
    pub help_visible_height: usize,
    pub last_deleted: Option<(NaiveDate, usize, Entry)>,
    pub last_filter_query: Option<String>,
    pub config: Config,
    pub active_journal: JournalSlot,
    pub in_git_repo: bool,
    pub hide_completed: bool,
}

impl App {
    pub fn new(config: Config) -> io::Result<Self> {
        let current_date = Local::now().date_naive();
        let lines = storage::load_day_lines(current_date)?;
        let entry_indices = Self::compute_entry_indices(&lines);
        let later_entries = storage::collect_later_entries_for_date(current_date)?;
        let active_journal = storage::get_active_slot();
        let in_git_repo = storage::find_git_root().is_some();

        Ok(Self {
            current_date,
            lines,
            entry_indices: entry_indices.clone(),
            view: ViewMode::Daily(DailyState::new(entry_indices.len(), later_entries)),
            input_mode: InputMode::Normal,
            edit_buffer: None,
            command_buffer: CursorBuffer::empty(),
            should_quit: false,
            status_message: None,
            show_help: false,
            help_scroll: 0,
            help_visible_height: 0,
            last_deleted: None,
            last_filter_query: None,
            config,
            active_journal,
            in_git_repo,
            hide_completed: false,
        })
    }

    #[must_use]
    pub fn compute_entry_indices(lines: &[Line]) -> Vec<usize> {
        lines
            .iter()
            .enumerate()
            .filter_map(|(i, line)| {
                if matches!(line, Line::Entry(_)) {
                    Some(i)
                } else {
                    None
                }
            })
            .collect()
    }

    // === Daily Entry Accessors ===

    fn get_daily_entry(&self, entry_index: usize) -> Option<&Entry> {
        let line_idx = self.entry_indices.get(entry_index)?;
        if let Line::Entry(entry) = &self.lines[*line_idx] {
            Some(entry)
        } else {
            None
        }
    }

    fn get_daily_entry_mut(&mut self, entry_index: usize) -> Option<&mut Entry> {
        let line_idx = *self.entry_indices.get(entry_index)?;
        if let Line::Entry(entry) = &mut self.lines[line_idx] {
            Some(entry)
        } else {
            None
        }
    }

    // === Later Entry Helpers ===

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some(msg.into());
    }

    /// Saves current day's lines to storage, displaying any error as a status message.
    pub fn save(&mut self) {
        if let Err(e) = storage::save_day_lines(self.current_date, &self.lines) {
            self.set_status(format!("Failed to save: {e}"));
        }
    }

    // === Navigation (unified) ===

    pub fn move_up(&mut self) {
        match &mut self.view {
            ViewMode::Daily(state) => {
                state.selected = state.selected.saturating_sub(1);
            }
            ViewMode::Filter(state) => {
                state.selected = state.selected.saturating_sub(1);
            }
        }
    }

    pub fn move_down(&mut self) {
        let total = self.visible_entry_count();
        if let ViewMode::Daily(state) = &mut self.view
            && total > 0
            && state.selected < total - 1
        {
            state.selected += 1;
        } else if let ViewMode::Filter(state) = &mut self.view
            && !state.entries.is_empty()
            && state.selected < state.entries.len() - 1
        {
            state.selected += 1;
        }
    }

    pub fn jump_to_first(&mut self) {
        match &mut self.view {
            ViewMode::Daily(state) => state.selected = 0,
            ViewMode::Filter(state) => state.selected = 0,
        }
    }

    pub fn jump_to_last(&mut self) {
        let total = self.visible_entry_count();
        if let ViewMode::Daily(state) = &mut self.view
            && total > 0
        {
            state.selected = total - 1;
        } else if let ViewMode::Filter(state) = &mut self.view
            && !state.entries.is_empty()
        {
            state.selected = state.entries.len() - 1;
        }
    }

    // === Unified Entry Operations ===

    /// Delete the currently selected entry (view-aware)
    pub fn delete_current_entry(&mut self) -> io::Result<()> {
        // Extract info from selected item before mutating
        let delete_info = match self.get_selected_item() {
            SelectedItem::Later { entry, .. } => Some(DeleteTarget::Later {
                source_date: entry.source_date,
                line_index: entry.line_index,
                entry_type: entry.entry_type.clone(),
                content: entry.content.clone(),
            }),
            SelectedItem::Daily { line_idx, entry, .. } => Some(DeleteTarget::Daily {
                line_idx,
                entry: entry.clone(),
            }),
            SelectedItem::Filter { index, entry } => Some(DeleteTarget::Filter {
                index,
                source_date: entry.source_date,
                line_index: entry.line_index,
                entry_type: entry.entry_type.clone(),
                content: entry.content.clone(),
            }),
            SelectedItem::None => None,
        };

        let Some(target) = delete_info else {
            return Ok(());
        };

        match target {
            DeleteTarget::Later {
                source_date,
                line_index,
                entry_type,
                content,
                ..
            } => {
                storage::delete_entry(source_date, line_index)?;
                self.last_deleted = Some((source_date, line_index, Entry { entry_type, content }));

                if let ViewMode::Daily(state) = &mut self.view {
                    state.later_entries =
                        storage::collect_later_entries_for_date(self.current_date)?;
                }

                let visible = self.visible_entry_count();
                if let ViewMode::Daily(state) = &mut self.view
                    && visible > 0
                    && state.selected >= visible
                {
                    state.selected = visible - 1;
                }
            }
            DeleteTarget::Daily { line_idx, entry } => {
                self.last_deleted = Some((self.current_date, line_idx, entry));
                self.lines.remove(line_idx);
                self.entry_indices = Self::compute_entry_indices(&self.lines);

                let visible = self.visible_entry_count();
                if let ViewMode::Daily(state) = &mut self.view
                    && visible > 0
                    && state.selected >= visible
                {
                    state.selected = visible - 1;
                }
                self.save();
            }
            DeleteTarget::Filter {
                index,
                source_date,
                line_index,
                entry_type,
                content,
            } => {
                self.last_deleted =
                    Some((source_date, line_index, Entry { entry_type, content }));
                storage::delete_entry(source_date, line_index)?;

                if let ViewMode::Filter(state) = &mut self.view {
                    state.entries.remove(index);

                    for filter_entry in &mut state.entries {
                        if filter_entry.source_date == source_date
                            && filter_entry.line_index > line_index
                        {
                            filter_entry.line_index -= 1;
                        }
                    }

                    if !state.entries.is_empty() && state.selected >= state.entries.len() {
                        state.selected = state.entries.len() - 1;
                    }
                }

                if source_date == self.current_date {
                    self.reload_current_day()?;
                }
            }
        }
        Ok(())
    }

    /// Toggle task completion (view-aware)
    pub fn toggle_current_entry(&mut self) -> io::Result<()> {
        let toggle_info = match self.get_selected_item() {
            SelectedItem::Later { entry, .. } => {
                if matches!(entry.entry_type, EntryType::Task { .. }) {
                    Some(ToggleTarget::Later {
                        source_date: entry.source_date,
                        line_index: entry.line_index,
                    })
                } else {
                    None
                }
            }
            SelectedItem::Daily { line_idx, entry, .. } => {
                if matches!(entry.entry_type, EntryType::Task { .. }) {
                    Some(ToggleTarget::Daily { line_idx })
                } else {
                    None
                }
            }
            SelectedItem::Filter { index, entry } => {
                if matches!(entry.entry_type, EntryType::Task { .. }) {
                    Some(ToggleTarget::Filter {
                        index,
                        source_date: entry.source_date,
                        line_index: entry.line_index,
                    })
                } else {
                    None
                }
            }
            SelectedItem::None => None,
        };

        let Some(target) = toggle_info else {
            return Ok(());
        };

        match target {
            ToggleTarget::Later {
                source_date,
                line_index,
            } => {
                storage::toggle_entry_complete(source_date, line_index)?;
                if let ViewMode::Daily(state) = &mut self.view {
                    state.later_entries =
                        storage::collect_later_entries_for_date(self.current_date)?;
                }
            }
            ToggleTarget::Daily { line_idx } => {
                if let Line::Entry(entry) = &mut self.lines[line_idx] {
                    entry.toggle_complete();
                    self.save();
                }
            }
            ToggleTarget::Filter {
                index,
                source_date,
                line_index,
            } => {
                storage::toggle_entry_complete(source_date, line_index)?;

                if let ViewMode::Filter(state) = &mut self.view {
                    let filter_entry = &mut state.entries[index];
                    filter_entry.completed = !filter_entry.completed;
                    if let EntryType::Task { completed } = &mut filter_entry.entry_type {
                        *completed = filter_entry.completed;
                    }
                }

                if source_date == self.current_date {
                    self.reload_current_day()?;
                }
            }
        }
        Ok(())
    }

    /// Start editing the current entry (view-aware)
    pub fn edit_current_entry(&mut self) {
        let (ctx, content) = match self.get_selected_item() {
            SelectedItem::Later { index, entry } => (
                EditContext::LaterEdit {
                    source_date: entry.source_date,
                    line_index: entry.line_index,
                    later_index: index,
                },
                entry.content.clone(),
            ),
            SelectedItem::Daily { index, entry, .. } => (
                EditContext::Daily { entry_index: index },
                entry.content.clone(),
            ),
            SelectedItem::Filter { index, entry } => (
                EditContext::FilterEdit {
                    date: entry.source_date,
                    line_index: entry.line_index,
                    filter_index: index,
                },
                entry.content.clone(),
            ),
            SelectedItem::None => return,
        };

        self.edit_buffer = Some(CursorBuffer::new(content));
        self.input_mode = InputMode::Edit(ctx);
    }

    pub fn yank_current_entry(&mut self) {
        let content = match self.get_selected_item() {
            SelectedItem::Later { entry, .. } => entry.content.clone(),
            SelectedItem::Daily { entry, .. } => entry.content.clone(),
            SelectedItem::Filter { entry, .. } => entry.content.clone(),
            SelectedItem::None => return,
        };

        match Self::copy_to_clipboard(&content) {
            Ok(()) => self.set_status("Yanked"),
            Err(e) => self.set_status(format!("Failed to yank: {e}")),
        }
    }

    fn copy_to_clipboard(text: &str) -> Result<(), arboard::Error> {
        let mut clipboard = arboard::Clipboard::new()?;
        clipboard.set_text(text)?;
        Ok(())
    }

    // === Edit Mode Operations ===

    /// Cycle entry type while editing (BackTab)
    pub fn cycle_edit_entry_type(&mut self) {
        match &mut self.input_mode {
            InputMode::Edit(EditContext::Daily { entry_index }) => {
                let line_idx = match self.entry_indices.get(*entry_index) {
                    Some(&idx) => idx,
                    None => return,
                };
                if let Line::Entry(entry) = &mut self.lines[line_idx] {
                    entry.entry_type = entry.entry_type.cycle();
                }
            }
            InputMode::Edit(EditContext::FilterEdit {
                date,
                line_index,
                filter_index,
            }) => {
                let date = *date;
                let line_index = *line_index;
                let filter_index = *filter_index;

                if let Ok(Some(new_type)) = storage::cycle_entry_type(date, line_index)
                    && let ViewMode::Filter(state) = &mut self.view
                    && let Some(filter_entry) = state.entries.get_mut(filter_index)
                {
                    filter_entry.entry_type = new_type;
                    filter_entry.completed =
                        matches!(filter_entry.entry_type, EntryType::Task { completed: true });
                    if date == self.current_date {
                        let _ = self.reload_current_day();
                    }
                }
            }
            InputMode::Edit(EditContext::FilterQuickAdd { entry_type, .. }) => {
                *entry_type = entry_type.cycle();
            }
            InputMode::Edit(EditContext::LaterEdit {
                source_date,
                line_index,
                later_index,
            }) => {
                let source_date = *source_date;
                let line_index = *line_index;
                let later_index = *later_index;

                if let Ok(Some(new_type)) = storage::cycle_entry_type(source_date, line_index)
                    && let ViewMode::Daily(state) = &mut self.view
                    && let Some(later_entry) = state.later_entries.get_mut(later_index)
                {
                    later_entry.entry_type = new_type;
                    later_entry.completed =
                        matches!(later_entry.entry_type, EntryType::Task { completed: true });
                }
            }
            _ => {}
        }
    }

    /// Save and exit edit mode (Enter)
    pub fn exit_edit(&mut self) {
        let Some(buffer) = self.edit_buffer.take() else {
            self.input_mode = InputMode::Normal;
            return;
        };
        let content = buffer.into_content();
        let content = storage::expand_favorite_tags(&content, &self.config.favorite_tags);
        let content = storage::normalize_natural_dates(&content, Local::now().date_naive());

        match std::mem::replace(&mut self.input_mode, InputMode::Normal) {
            InputMode::Edit(EditContext::Daily { entry_index }) => {
                if content.trim().is_empty() {
                    self.delete_at_index_daily(entry_index);
                    if let ViewMode::Daily(state) = &mut self.view {
                        state.scroll_offset = 0;
                    }
                } else if let Some(entry) = self.get_daily_entry_mut(entry_index) {
                    entry.content = content;
                    self.save();
                }
            }
            InputMode::Edit(EditContext::FilterEdit {
                date, line_index, ..
            }) => {
                if content.trim().is_empty() {
                    let _ = storage::delete_entry(date, line_index);
                } else {
                    match storage::update_entry_content(date, line_index, content) {
                        Ok(false) => {
                            self.set_status(format!(
                                "Failed to update: no entry at index {line_index} for {date}"
                            ));
                        }
                        Err(e) => {
                            self.set_status(format!("Failed to save: {e}"));
                        }
                        Ok(true) => {}
                    }
                }
                if date == self.current_date {
                    let _ = self.reload_current_day();
                }
                let _ = self.refresh_filter();
            }
            InputMode::Edit(EditContext::FilterQuickAdd { date, entry_type }) => {
                if !content.trim().is_empty()
                    && let Ok(mut lines) = storage::load_day_lines(date)
                {
                    let entry = Entry {
                        entry_type,
                        content,
                    };
                    lines.push(Line::Entry(entry));
                    let _ = storage::save_day_lines(date, &lines);
                    if date == self.current_date {
                        let _ = self.reload_current_day();
                    }
                }
                let _ = self.refresh_filter();
                if let ViewMode::Filter(state) = &mut self.view {
                    state.selected = state.entries.len().saturating_sub(1);
                }
            }
            InputMode::Edit(EditContext::LaterEdit {
                source_date,
                line_index,
                ..
            }) => {
                if content.trim().is_empty() {
                    let _ = storage::delete_entry(source_date, line_index);
                } else {
                    match storage::update_entry_content(source_date, line_index, content) {
                        Ok(false) => {
                            self.set_status(format!(
                                "Failed to update: no entry at index {line_index} for {source_date}"
                            ));
                        }
                        Err(e) => {
                            self.set_status(format!("Failed to save: {e}"));
                        }
                        Ok(true) => {}
                    }
                }
                if let ViewMode::Daily(state) = &mut self.view {
                    state.later_entries =
                        storage::collect_later_entries_for_date(self.current_date)
                            .unwrap_or_default();
                }
            }
            _ => {}
        }
    }

    /// Cancel edit mode without saving (Esc)
    pub fn cancel_edit(&mut self) {
        self.edit_buffer = None;

        match std::mem::replace(&mut self.input_mode, InputMode::Normal) {
            InputMode::Edit(EditContext::Daily { entry_index }) => {
                if let Some(entry) = self.get_daily_entry(entry_index)
                    && entry.content.is_empty()
                {
                    self.delete_at_index_daily(entry_index);
                    if let ViewMode::Daily(state) = &mut self.view {
                        state.scroll_offset = 0;
                    }
                }
            }
            InputMode::Edit(EditContext::FilterEdit { .. })
            | InputMode::Edit(EditContext::FilterQuickAdd { .. }) => {
                // Just return to filter view, no cleanup needed
            }
            InputMode::Edit(EditContext::LaterEdit { .. }) => {
                // Just return to daily view, no cleanup needed
            }
            _ => {}
        }
    }

    /// Save and add new entry (Tab)
    pub fn commit_and_add_new(&mut self) {
        let Some(buffer) = self.edit_buffer.take() else {
            return;
        };
        let content = buffer.into_content();

        match std::mem::replace(&mut self.input_mode, InputMode::Normal) {
            InputMode::Edit(EditContext::FilterQuickAdd { date, entry_type }) => {
                if !content.trim().is_empty()
                    && let Ok(mut lines) = storage::load_day_lines(date)
                {
                    let line_index = lines.len();
                    let entry = Entry {
                        entry_type: entry_type.clone(),
                        content: content.clone(),
                    };
                    lines.push(Line::Entry(entry));
                    let _ = storage::save_day_lines(date, &lines);
                    if date == self.current_date {
                        let _ = self.reload_current_day();
                    }

                    // Add to filter entries without refreshing (defer filter until exit)
                    if let ViewMode::Filter(state) = &mut self.view {
                        state.entries.push(FilterEntry {
                            source_date: date,
                            line_index,
                            entry_type: entry_type.clone(),
                            content,
                            completed: matches!(entry_type, EntryType::Task { completed: true }),
                        });
                        state.selected = state.entries.len().saturating_sub(1);
                    }
                }
                self.edit_buffer = Some(CursorBuffer::empty());
                self.input_mode = InputMode::Edit(EditContext::FilterQuickAdd {
                    date,
                    entry_type: match entry_type {
                        EntryType::Task { .. } => EntryType::Task { completed: false },
                        other => other,
                    },
                });
            }
            InputMode::Edit(EditContext::Daily { entry_index }) => {
                if content.trim().is_empty() {
                    let was_at_end = entry_index == self.entry_indices.len().saturating_sub(1);
                    self.delete_at_index_daily(entry_index);
                    if let ViewMode::Daily(state) = &mut self.view
                        && !was_at_end
                        && state.selected > 0
                    {
                        state.selected -= 1;
                    }
                    return;
                }

                let entry_type = self
                    .get_daily_entry(entry_index)
                    .map(|e| e.entry_type.clone())
                    .unwrap_or(EntryType::Task { completed: false });

                if let Some(entry) = self.get_daily_entry_mut(entry_index) {
                    entry.content = content;
                }
                self.save();

                let new_entry = Entry {
                    entry_type: match entry_type {
                        EntryType::Task { .. } => EntryType::Task { completed: false },
                        other => other,
                    },
                    content: String::new(),
                };
                self.add_entry_internal(new_entry, InsertPosition::Below);
            }
            _ => {}
        }
    }

    // === Daily-Only Operations ===

    fn delete_at_index_daily(&mut self, entry_index: usize) {
        if entry_index >= self.entry_indices.len() {
            return;
        }
        let line_idx = self.entry_indices[entry_index];
        if let Line::Entry(entry) = &self.lines[line_idx] {
            self.last_deleted = Some((self.current_date, line_idx, entry.clone()));
        }
        self.lines.remove(line_idx);
        self.entry_indices = Self::compute_entry_indices(&self.lines);

        let visible = self.visible_entry_count();
        if let ViewMode::Daily(state) = &mut self.view
            && visible > 0
            && state.selected >= visible
        {
            state.selected = visible - 1;
        }
    }

    fn add_entry_internal(&mut self, entry: Entry, position: InsertPosition) {
        let insert_pos = if matches!(position, InsertPosition::Bottom) || self.entry_indices.is_empty()
        {
            self.lines.len()
        } else {
            match self.get_selected_item() {
                SelectedItem::Daily { index, .. } => match position {
                    InsertPosition::Below => self.entry_indices[index] + 1,
                    InsertPosition::Above => self.entry_indices[index],
                    InsertPosition::Bottom => unreachable!(),
                },
                _ => self.lines.len(),
            }
        };

        self.lines.insert(insert_pos, Line::Entry(entry));
        self.entry_indices = Self::compute_entry_indices(&self.lines);

        let entry_index = self
            .entry_indices
            .iter()
            .position(|&idx| idx == insert_pos)
            .unwrap_or(self.entry_indices.len().saturating_sub(1));

        let visible_index = self.actual_to_visible_index(entry_index);
        if let ViewMode::Daily(state) = &mut self.view {
            state.selected = visible_index;
        }

        self.edit_buffer = Some(CursorBuffer::empty());
        self.input_mode = InputMode::Edit(EditContext::Daily { entry_index });
    }

    pub fn new_task(&mut self, position: InsertPosition) {
        self.add_entry_internal(Entry::new_task(""), position);
    }

    pub fn undo(&mut self) {
        let Some((date, line_idx, entry)) = self.last_deleted.take() else {
            return;
        };

        match &self.view {
            ViewMode::Daily(_) => {
                // Only undo if we're on the same day
                if date != self.current_date {
                    self.set_status(format!(
                        "Undo: entry was from {}, go to that day first",
                        date.format("%m/%d")
                    ));
                    self.last_deleted = Some((date, line_idx, entry));
                    return;
                }
                let insert_idx = line_idx.min(self.lines.len());
                let is_completed = matches!(entry.entry_type, EntryType::Task { completed: true });
                self.lines.insert(insert_idx, Line::Entry(entry));
                self.entry_indices = Self::compute_entry_indices(&self.lines);

                // If restored entry would be hidden, turn off hide mode
                if self.hide_completed && is_completed {
                    self.hide_completed = false;
                }

                let visible_idx = self
                    .entry_indices
                    .iter()
                    .position(|&i| i == insert_idx)
                    .map(|actual_idx| self.actual_to_visible_index(actual_idx));

                if let ViewMode::Daily(state) = &mut self.view
                    && let Some(idx) = visible_idx
                {
                    state.selected = idx;
                }
                self.save();
            }
            ViewMode::Filter(_) => {
                if let Ok(mut lines) = storage::load_day_lines(date) {
                    let insert_idx = line_idx.min(lines.len());
                    lines.insert(insert_idx, Line::Entry(entry.clone()));
                    let _ = storage::save_day_lines(date, &lines);

                    let filter_entry = FilterEntry {
                        source_date: date,
                        line_index: insert_idx,
                        entry_type: entry.entry_type.clone(),
                        content: entry.content,
                        completed: matches!(entry.entry_type, EntryType::Task { completed: true }),
                    };

                    if let ViewMode::Filter(state) = &mut self.view {
                        state.entries.push(filter_entry);
                        state.selected = state.entries.len() - 1;
                    }

                    if date == self.current_date {
                        let _ = self.reload_current_day();
                    }
                }
            }
        }
    }

    pub fn sort_entries(&mut self) {
        let entry_positions: Vec<usize> = self
            .lines
            .iter()
            .enumerate()
            .filter_map(|(i, l)| matches!(l, Line::Entry(_)).then_some(i))
            .collect();

        if entry_positions.is_empty() {
            return;
        }

        let sort_order = self.config.validated_sort_order();
        let get_priority = |line: &Line| -> usize {
            let Line::Entry(entry) = line else {
                return sort_order.len();
            };
            for (i, type_name) in sort_order.iter().enumerate() {
                match (type_name.as_str(), &entry.entry_type) {
                    ("completed", EntryType::Task { completed: true }) => return i,
                    ("uncompleted", EntryType::Task { completed: false }) => return i,
                    ("notes", EntryType::Note) => return i,
                    ("events", EntryType::Event) => return i,
                    _ => {}
                }
            }
            sort_order.len()
        };

        let mut entries: Vec<Line> = entry_positions
            .iter()
            .map(|&i| self.lines[i].clone())
            .collect();

        entries.sort_by_key(|line| get_priority(line));

        for (pos, entry) in entry_positions.iter().zip(entries.into_iter()) {
            self.lines[*pos] = entry;
        }

        self.entry_indices = Self::compute_entry_indices(&self.lines);
        self.save();
    }

    pub fn toggle_hide_completed(&mut self) {
        self.hide_completed = !self.hide_completed;

        if self.hide_completed {
            self.clamp_selection_to_visible();
        }
    }

    fn clamp_selection_to_visible(&mut self) {
        let visible_count = self.visible_entry_count();

        let ViewMode::Daily(state) = &mut self.view else {
            return;
        };

        if visible_count == 0 {
            state.selected = 0;
            state.scroll_offset = 0;
        } else if state.selected >= visible_count {
            state.selected = visible_count.saturating_sub(1);
        }
        state.scroll_offset = 0;
    }

    /// Returns the currently selected item, accounting for hidden completed entries.
    #[must_use]
    pub fn get_selected_item(&self) -> SelectedItem<'_> {
        match &self.view {
            ViewMode::Daily(state) => {
                let mut visible_idx = 0;

                for (actual_idx, later_entry) in state.later_entries.iter().enumerate() {
                    if self.hide_completed && later_entry.completed {
                        continue;
                    }
                    if visible_idx == state.selected {
                        return SelectedItem::Later {
                            index: actual_idx,
                            entry: later_entry,
                        };
                    }
                    visible_idx += 1;
                }

                for (actual_idx, &line_idx) in self.entry_indices.iter().enumerate() {
                    if let Line::Entry(entry) = &self.lines[line_idx] {
                        if self.hide_completed
                            && matches!(entry.entry_type, EntryType::Task { completed: true })
                        {
                            continue;
                        }
                        if visible_idx == state.selected {
                            return SelectedItem::Daily {
                                index: actual_idx,
                                line_idx,
                                entry,
                            };
                        }
                        visible_idx += 1;
                    }
                }

                SelectedItem::None
            }
            ViewMode::Filter(state) => match state.entries.get(state.selected) {
                Some(entry) => SelectedItem::Filter {
                    index: state.selected,
                    entry,
                },
                None => SelectedItem::None,
            },
        }
    }

    #[must_use]
    pub fn visible_entry_count(&self) -> usize {
        let ViewMode::Daily(state) = &self.view else {
            return 0;
        };

        if !self.hide_completed {
            return state.later_entries.len() + self.entry_indices.len();
        }

        let visible_later = state.later_entries.iter().filter(|e| !e.completed).count();
        let visible_regular = self
            .entry_indices
            .iter()
            .filter(|&&i| {
                if let Line::Entry(entry) = &self.lines[i] {
                    !matches!(entry.entry_type, EntryType::Task { completed: true })
                } else {
                    true
                }
            })
            .count();
        visible_later + visible_regular
    }

    #[must_use]
    pub fn hidden_completed_count(&self) -> usize {
        let ViewMode::Daily(state) = &self.view else {
            return 0;
        };

        let hidden_later = state.later_entries.iter().filter(|e| e.completed).count();
        let hidden_regular = self
            .entry_indices
            .iter()
            .filter(|&&i| {
                if let Line::Entry(entry) = &self.lines[i] {
                    matches!(entry.entry_type, EntryType::Task { completed: true })
                } else {
                    false
                }
            })
            .count();
        hidden_later + hidden_regular
    }

    /// Converts an actual entry index to a visible index (accounting for later entries and hidden completed)
    #[must_use]
    fn actual_to_visible_index(&self, actual_entry_idx: usize) -> usize {
        let ViewMode::Daily(state) = &self.view else {
            return 0;
        };

        let visible_later = if self.hide_completed {
            state.later_entries.iter().filter(|e| !e.completed).count()
        } else {
            state.later_entries.len()
        };

        let visible_before = self.entry_indices[..actual_entry_idx]
            .iter()
            .filter(|&&i| {
                if self.hide_completed {
                    if let Line::Entry(entry) = &self.lines[i] {
                        !matches!(entry.entry_type, EntryType::Task { completed: true })
                    } else {
                        true
                    }
                } else {
                    true
                }
            })
            .count();

        visible_later + visible_before
    }

    pub fn enter_reorder_mode(&mut self) {
        // Block reorder mode if a later entry is selected
        if matches!(self.get_selected_item(), SelectedItem::Later { .. }) {
            self.set_status("Cannot reorder later entries");
            return;
        }

        if let ViewMode::Daily(state) = &mut self.view
            && !self.entry_indices.is_empty()
        {
            state.original_lines = Some(self.lines.clone());
            self.input_mode = InputMode::Reorder;
        }
    }

    pub fn exit_reorder_mode(&mut self, save: bool) {
        if !matches!(self.view, ViewMode::Daily(_)) {
            return;
        }
        if save {
            self.save();
        } else if let ViewMode::Daily(state) = &mut self.view
            && let Some(original) = state.original_lines.take()
        {
            self.lines = original;
            self.entry_indices = Self::compute_entry_indices(&self.lines);
        }
        if let ViewMode::Daily(state) = &mut self.view {
            state.original_lines = None;
        }
        self.input_mode = InputMode::Normal;
    }

    pub fn reorder_move_up(&mut self) {
        let SelectedItem::Daily {
            index: curr_entry_idx,
            ..
        } = self.get_selected_item()
        else {
            return;
        };

        // Find previous visible entry
        let prev_entry_idx = self.entry_indices[..curr_entry_idx]
            .iter()
            .enumerate()
            .rev()
            .find(|&(_, &line_idx)| {
                if self.hide_completed {
                    if let Line::Entry(e) = &self.lines[line_idx] {
                        !matches!(e.entry_type, EntryType::Task { completed: true })
                    } else {
                        true
                    }
                } else {
                    true
                }
            })
            .map(|(idx, _)| idx);

        let Some(prev_entry_idx) = prev_entry_idx else {
            return; // Already at top
        };

        // Remove current entry and insert before previous visible entry
        let curr_line_idx = self.entry_indices[curr_entry_idx];
        let prev_line_idx = self.entry_indices[prev_entry_idx];
        let entry = self.lines.remove(curr_line_idx);
        self.lines.insert(prev_line_idx, entry);
        self.entry_indices = Self::compute_entry_indices(&self.lines);

        if let ViewMode::Daily(state) = &mut self.view {
            state.selected -= 1;
        }
    }

    pub fn reorder_move_down(&mut self) {
        let SelectedItem::Daily {
            index: curr_entry_idx,
            ..
        } = self.get_selected_item()
        else {
            return;
        };

        // Find next visible entry
        let next_entry_idx = self.entry_indices[curr_entry_idx + 1..]
            .iter()
            .enumerate()
            .find(|&(_, &line_idx)| {
                if self.hide_completed {
                    if let Line::Entry(e) = &self.lines[line_idx] {
                        !matches!(e.entry_type, EntryType::Task { completed: true })
                    } else {
                        true
                    }
                } else {
                    true
                }
            })
            .map(|(offset, _)| curr_entry_idx + 1 + offset);

        let Some(next_entry_idx) = next_entry_idx else {
            return; // Already at bottom
        };

        // Remove current entry and insert after next visible entry
        let curr_line_idx = self.entry_indices[curr_entry_idx];
        let next_line_idx = self.entry_indices[next_entry_idx];
        let entry = self.lines.remove(curr_line_idx);
        // After removal, next_line_idx shifted down by 1 if it was after curr
        let insert_pos = if next_line_idx > curr_line_idx {
            next_line_idx // Was after, now at next_line_idx - 1, insert after = next_line_idx
        } else {
            next_line_idx + 1
        };
        self.lines.insert(insert_pos, entry);
        self.entry_indices = Self::compute_entry_indices(&self.lines);

        if let ViewMode::Daily(state) = &mut self.view {
            state.selected += 1;
        }
    }

    // === Day Navigation (Daily only) ===

    /// Load a day's data into self, returning later entries for view construction.
    fn load_day(&mut self, date: NaiveDate) -> io::Result<Vec<LaterEntry>> {
        self.current_date = date;
        self.lines = storage::load_day_lines(date)?;
        self.entry_indices = Self::compute_entry_indices(&self.lines);
        storage::collect_later_entries_for_date(date)
    }

    pub fn goto_day(&mut self, date: NaiveDate) -> io::Result<()> {
        if date == self.current_date {
            return Ok(());
        }

        self.save();
        let later_entries = self.load_day(date)?;
        self.edit_buffer = None;
        self.view = ViewMode::Daily(DailyState::new(self.entry_indices.len(), later_entries));

        if self.hide_completed {
            self.clamp_selection_to_visible();
        }
        self.input_mode = InputMode::Normal;

        Ok(())
    }

    pub fn prev_day(&mut self) -> io::Result<()> {
        if let Some(prev) = self.current_date.checked_sub_days(Days::new(1)) {
            self.goto_day(prev)?;
        }
        Ok(())
    }

    pub fn next_day(&mut self) -> io::Result<()> {
        if let Some(next) = self.current_date.checked_add_days(Days::new(1)) {
            self.goto_day(next)?;
        }
        Ok(())
    }

    pub fn goto_today(&mut self) -> io::Result<()> {
        self.goto_day(Local::now().date_naive())
    }

    #[must_use]
    pub fn parse_goto_date(input: &str) -> Option<NaiveDate> {
        // YYYY/MM/DD (only if first part is exactly 4 digits)
        if let Some(first_slash) = input.find('/')
            && first_slash == 4
            && input[..4].chars().all(|c| c.is_ascii_digit())
            && let Ok(date) = NaiveDate::parse_from_str(input, "%Y/%m/%d")
        {
            return Some(date);
        }

        let parts: Vec<&str> = input.split('/').collect();

        // MM/DD/YYYY or MM/DD/YY
        if parts.len() == 3
            && let (Ok(month), Ok(day), Ok(year)) = (
                parts[0].parse::<u32>(),
                parts[1].parse::<u32>(),
                parts[2].parse::<i32>(),
            )
        {
            let full_year = if year < 100 { 2000 + year } else { year };
            return NaiveDate::from_ymd_opt(full_year, month, day);
        }

        // MM/DD (always future - if date passed this year, use next year)
        if parts.len() == 2
            && let (Ok(month), Ok(day)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>())
        {
            let today = Local::now().date_naive();
            if let Some(date) = NaiveDate::from_ymd_opt(today.year(), month, day) {
                if date >= today {
                    return Some(date);
                }
                return NaiveDate::from_ymd_opt(today.year() + 1, month, day);
            }
        }

        None
    }

    pub fn open_journal(&mut self, path: &str) -> io::Result<()> {
        self.save();

        let path = resolve_path(path);
        storage::set_project_path(path.clone());
        storage::set_active_slot(storage::JournalSlot::Project);
        self.active_journal = JournalSlot::Project;
        let later_entries = self.load_day(Local::now().date_naive())?;
        self.view = ViewMode::Daily(DailyState::new(self.entry_indices.len(), later_entries));
        self.set_status(format!("Opened: {}", path.display()));
        Ok(())
    }

    fn switch_to_journal(&mut self, slot: JournalSlot) -> io::Result<()> {
        if self.active_journal == slot {
            return Ok(());
        }
        self.save();
        storage::set_active_slot(slot);
        self.active_journal = slot;
        let later_entries = self.load_day(Local::now().date_naive())?;
        self.view = ViewMode::Daily(DailyState::new(self.entry_indices.len(), later_entries));
        self.set_status(match slot {
            JournalSlot::Global => "Switched to Global journal",
            JournalSlot::Project => "Switched to Project journal",
        });
        Ok(())
    }

    pub fn switch_to_global(&mut self) -> io::Result<()> {
        self.switch_to_journal(JournalSlot::Global)
    }

    pub fn switch_to_project(&mut self) -> io::Result<()> {
        self.switch_to_journal(JournalSlot::Project)
    }

    pub fn toggle_journal(&mut self) -> io::Result<()> {
        match self.active_journal {
            JournalSlot::Global => {
                // Check if project journal exists
                if storage::get_project_path().is_some() {
                    self.switch_to_project()?;
                } else if self.in_git_repo {
                    // Prompt to create project journal
                    self.input_mode = InputMode::Confirm(ConfirmContext::CreateProjectJournal);
                } else {
                    self.set_status("Not in a git repository - no project journal available");
                }
            }
            JournalSlot::Project => {
                self.switch_to_global()?;
            }
        }
        Ok(())
    }

    pub fn reload_config(&mut self) -> io::Result<()> {
        self.config = Config::load()?;
        self.set_status("Config reloaded");
        Ok(())
    }

    // === Command Mode ===

    pub fn execute_command(&mut self) -> io::Result<()> {
        let cmd = self.command_buffer.content().to_string();
        self.command_buffer.clear();
        let parts: Vec<&str> = cmd.trim().splitn(2, ' ').collect();
        let command = parts.first().copied().unwrap_or("");
        let arg = parts.get(1).copied().unwrap_or("").trim();

        match command {
            "q" | "quit" => {
                self.save();
                self.should_quit = true;
            }
            "goto" | "g" => {
                if arg.is_empty() {
                    self.set_status("Usage: :goto YYYY/MM/DD or :goto MM/DD");
                } else if let Some(date) = Self::parse_goto_date(arg) {
                    self.goto_day(date)?;
                } else {
                    self.set_status(format!("Invalid date: {arg}"));
                }
            }
            "o" | "open" => {
                if arg.is_empty() {
                    self.set_status("Usage: :open /path/to/file.md");
                } else {
                    self.open_journal(arg)?;
                }
            }
            "config-reload" => {
                self.reload_config()?;
            }
            "global" => {
                self.switch_to_global()?;
            }
            "project" => {
                if arg.is_empty() {
                    // Switch to project journal (prompt to create if needed)
                    if storage::get_project_path().is_some() {
                        self.switch_to_project()?;
                    } else if self.in_git_repo {
                        self.input_mode = InputMode::Confirm(ConfirmContext::CreateProjectJournal);
                        return Ok(());
                    } else {
                        self.set_status("Not in a git repository - no project journal available");
                    }
                } else {
                    // :project /path/to.md - set and switch to that path
                    self.open_journal(arg)?;
                }
            }
            "init-project" => {
                if storage::get_project_path().is_some() {
                    self.set_status("Project journal already exists");
                } else if self.in_git_repo {
                    // In git repo: use the confirm flow (creates at git root, asks about .gitignore)
                    self.input_mode = InputMode::Confirm(ConfirmContext::CreateProjectJournal);
                    return Ok(());
                } else {
                    // Not in git repo: create in cwd, no .gitignore prompt
                    let cwd = std::env::current_dir()?;
                    let caliber_dir = cwd.join(".caliber");
                    std::fs::create_dir_all(&caliber_dir)?;
                    let journal_path = caliber_dir.join("journal.md");
                    if !journal_path.exists() {
                        std::fs::write(&journal_path, "")?;
                    }
                    storage::set_project_path(journal_path);
                    self.switch_to_project()?;
                    self.set_status("Project journal created");
                }
            }
            _ => {}
        }
        self.input_mode = InputMode::Normal;
        Ok(())
    }

    // === Filter Operations ===

    pub fn enter_filter_input(&mut self) {
        match &mut self.view {
            ViewMode::Filter(state) => {
                // Pre-fill with current query for modification
                state.query_buffer.set_content(&state.query);
            }
            ViewMode::Daily(_) => {
                // Use command_buffer temporarily, ensure it's clean
                self.command_buffer.clear();
            }
        }

        self.input_mode = InputMode::QueryInput;
    }

    pub fn execute_filter(&mut self) -> io::Result<()> {
        self.save();

        let query = match &mut self.view {
            ViewMode::Filter(state) => {
                let q = state.query_buffer.content().to_string();
                state.query_buffer.clear();
                q
            }
            ViewMode::Daily(_) => {
                let q = self.command_buffer.content().to_string();
                self.command_buffer.clear();
                q
            }
        };

        let (query, unknown_filters) = storage::expand_saved_filters(&query, &self.config.filters);
        let mut filter = storage::parse_filter_query(&query);
        filter.invalid_tokens.extend(unknown_filters);

        if !filter.invalid_tokens.is_empty() {
            self.set_status(format!(
                "Unknown filter: {}",
                filter.invalid_tokens.join(", ")
            ));
        }

        let entries = storage::collect_filtered_entries(&filter)?;
        let selected = entries.len().saturating_sub(1);

        self.view = ViewMode::Filter(FilterState {
            query,
            query_buffer: CursorBuffer::empty(),
            entries,
            selected,
            scroll_offset: 0,
        });
        self.input_mode = InputMode::Normal;
        Ok(())
    }

    pub fn quick_filter(&mut self, query: &str) -> io::Result<()> {
        self.save();
        let (query, unknown_filters) = storage::expand_saved_filters(query, &self.config.filters);
        let mut filter = storage::parse_filter_query(&query);
        filter.invalid_tokens.extend(unknown_filters);

        if !filter.invalid_tokens.is_empty() {
            self.set_status(format!(
                "Unknown filter: {}",
                filter.invalid_tokens.join(", ")
            ));
        }

        let entries = storage::collect_filtered_entries(&filter)?;
        let selected = entries.len().saturating_sub(1);

        self.view = ViewMode::Filter(FilterState {
            query,
            query_buffer: CursorBuffer::empty(),
            entries,
            selected,
            scroll_offset: 0,
        });
        self.input_mode = InputMode::Normal;
        Ok(())
    }

    pub fn cancel_filter_input(&mut self) {
        match &mut self.view {
            ViewMode::Filter(state) => {
                state.query_buffer.clear();
            }
            ViewMode::Daily(_) => {
                self.command_buffer.clear();
            }
        }
        self.input_mode = InputMode::Normal;
    }

    pub fn exit_filter(&mut self) {
        if let ViewMode::Filter(state) = &self.view {
            self.last_filter_query = Some(state.query.clone());
        }
        let later_entries =
            storage::collect_later_entries_for_date(self.current_date).unwrap_or_default();
        self.view = ViewMode::Daily(DailyState::new(self.entry_indices.len(), later_entries));
        self.input_mode = InputMode::Normal;
    }

    pub fn return_to_filter(&mut self) -> io::Result<()> {
        let query = self
            .last_filter_query
            .clone()
            .unwrap_or_else(|| self.config.default_filter.clone());
        self.quick_filter(&query)
    }

    pub fn refresh_filter(&mut self) -> io::Result<()> {
        let ViewMode::Filter(state) = &mut self.view else {
            return Ok(());
        };

        let filter = storage::parse_filter_query(&state.query);

        if !filter.invalid_tokens.is_empty() {
            self.status_message = Some(format!(
                "Unknown filter: {}",
                filter.invalid_tokens.join(", ")
            ));
        }

        state.entries = storage::collect_filtered_entries(&filter)?;
        state.selected = state.selected.min(state.entries.len().saturating_sub(1));
        state.scroll_offset = 0;
        Ok(())
    }

    /// Navigate to a specific day and select the entry at the given line index
    fn goto_entry_source(&mut self, date: NaiveDate, line_index: usize) -> io::Result<()> {
        if let ViewMode::Filter(state) = &self.view {
            self.last_filter_query = Some(state.query.clone());
        }
        if date != self.current_date {
            self.save();
        }

        let later_entries = self.load_day(date)?;
        let later_count = later_entries.len();

        let entry_pos = self
            .entry_indices
            .iter()
            .position(|&i| i == line_index)
            .unwrap_or(0);
        let selected = later_count + entry_pos;

        self.view = ViewMode::Daily(DailyState {
            selected,
            scroll_offset: 0,
            original_lines: None,
            later_entries,
        });
        self.input_mode = InputMode::Normal;
        self.edit_buffer = None;

        Ok(())
    }

    /// View the source day of the currently selected entry (unified across views)
    /// - Filter view: jumps to the source day of the selected filter entry
    /// - Daily view with later entry selected: jumps to the source day
    /// - Daily view with regular entry: no-op (already on source day)
    pub fn view_entry_source(&mut self) -> io::Result<()> {
        match self.get_selected_item() {
            SelectedItem::Filter { entry, .. } => {
                self.goto_entry_source(entry.source_date, entry.line_index)
            }
            SelectedItem::Later { entry, .. } => {
                self.goto_entry_source(entry.source_date, entry.line_index)
            }
            SelectedItem::Daily { .. } | SelectedItem::None => Ok(()),
        }
    }

    pub fn filter_quick_add(&mut self) {
        let today = Local::now().date_naive();
        self.edit_buffer = Some(CursorBuffer::empty());
        self.input_mode = InputMode::Edit(EditContext::FilterQuickAdd {
            date: today,
            entry_type: EntryType::Task { completed: false },
        });
    }

    // === Filter View Helpers ===

    #[must_use]
    pub fn filter_visual_line(&self) -> usize {
        let ViewMode::Filter(state) = &self.view else {
            return 0;
        };
        state.selected + FILTER_HEADER_LINES
    }

    #[must_use]
    pub fn filter_total_lines(&self) -> usize {
        let ViewMode::Filter(state) = &self.view else {
            return 1;
        };
        state.entries.len() + FILTER_HEADER_LINES
    }

    fn reload_current_day(&mut self) -> io::Result<()> {
        self.lines = storage::load_day_lines(self.current_date)?;
        self.entry_indices = Self::compute_entry_indices(&self.lines);
        Ok(())
    }
}
