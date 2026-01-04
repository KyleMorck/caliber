use std::io;

use crate::cursor::CursorBuffer;
use crate::storage::{self, Entry, EntryType};

use super::{App, EditContext, InputMode, Line, SelectedItem, ViewMode};

/// Internal helper for delete operations (owns data extracted from SelectedItem)
enum DeleteTarget {
    Later {
        source_date: chrono::NaiveDate,
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
        source_date: chrono::NaiveDate,
        line_index: usize,
        entry_type: EntryType,
        content: String,
    },
}

/// Internal helper for toggle operations
enum ToggleTarget {
    Later {
        source_date: chrono::NaiveDate,
        line_index: usize,
    },
    Daily {
        line_idx: usize,
    },
    Filter {
        index: usize,
        source_date: chrono::NaiveDate,
        line_index: usize,
    },
}

impl App {
    fn clamp_daily_selection(&mut self) {
        let visible = self.visible_entry_count();
        if let ViewMode::Daily(state) = &mut self.view
            && visible > 0
            && state.selected >= visible
        {
            state.selected = visible - 1;
        }
    }

    /// Delete the currently selected entry (view-aware)
    pub fn delete_current_entry(&mut self) -> io::Result<()> {
        let delete_info = match self.get_selected_item() {
            SelectedItem::Later { entry, .. } => Some(DeleteTarget::Later {
                source_date: entry.source_date,
                line_index: entry.line_index,
                entry_type: entry.entry_type.clone(),
                content: entry.content.clone(),
            }),
            SelectedItem::Daily {
                line_idx, entry, ..
            } => Some(DeleteTarget::Daily {
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
                self.last_deleted = Some((
                    source_date,
                    line_index,
                    Entry {
                        entry_type,
                        content,
                    },
                ));

                if let ViewMode::Daily(state) = &mut self.view {
                    state.later_entries =
                        storage::collect_later_entries_for_date(self.current_date)?;
                }
                self.clamp_daily_selection();
            }
            DeleteTarget::Daily { line_idx, entry } => {
                self.last_deleted = Some((self.current_date, line_idx, entry));
                self.lines.remove(line_idx);
                self.entry_indices = Self::compute_entry_indices(&self.lines);
                self.clamp_daily_selection();
                self.save();
            }
            DeleteTarget::Filter {
                index,
                source_date,
                line_index,
                entry_type,
                content,
            } => {
                self.last_deleted = Some((
                    source_date,
                    line_index,
                    Entry {
                        entry_type,
                        content,
                    },
                ));
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
        self.refresh_tag_cache();
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
            SelectedItem::Daily {
                line_idx, entry, ..
            } => {
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

    pub(super) fn delete_at_index_daily(&mut self, entry_index: usize) {
        if entry_index >= self.entry_indices.len() {
            return;
        }
        let line_idx = self.entry_indices[entry_index];
        if let Line::Entry(entry) = &self.lines[line_idx]
            && !entry.content.trim().is_empty()
        {
            self.last_deleted = Some((self.current_date, line_idx, entry.clone()));
        }
        self.lines.remove(line_idx);
        self.entry_indices = Self::compute_entry_indices(&self.lines);
        self.clamp_daily_selection();
    }
}
