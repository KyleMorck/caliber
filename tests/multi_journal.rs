mod helpers;

use chrono::NaiveDate;
use std::fs;
use tempfile::TempDir;

use caliber::app::App;
use caliber::config::Config;
use caliber::storage::{self, JournalSlot, Line};

/// MJ-4: Journal isolation - entries don't cross journals
#[test]
fn test_journal_isolation() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let temp_dir = TempDir::new().unwrap();

    let global_path = temp_dir.path().join("global.md");
    let project_path = temp_dir.path().join("project.md");

    fs::write(&global_path, "# 2026/01/15\n- [ ] Global entry\n").unwrap();
    fs::write(&project_path, "# 2026/01/15\n- [ ] Project entry\n").unwrap();

    storage::reset_journal_context();
    storage::set_journal_context(global_path, Some(project_path), JournalSlot::Global);

    let config = Config::default();
    let app = App::new_with_date(config, date).unwrap();

    // Should see global entry
    let has_global = app.entry_indices.iter().any(|&i| {
        if let Line::Entry(e) = &app.lines[i] {
            e.content.contains("Global entry")
        } else {
            false
        }
    });
    assert!(has_global, "Global entry should be visible");

    // Should NOT see project entry
    let has_project = app.entry_indices.iter().any(|&i| {
        if let Line::Entry(e) = &app.lines[i] {
            e.content.contains("Project entry")
        } else {
            false
        }
    });
    assert!(!has_project, "Project entry should not be visible in global");
}

/// MJ-4: Switch to project journal sees project entries
#[test]
fn test_project_journal_switch() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let temp_dir = TempDir::new().unwrap();

    let global_path = temp_dir.path().join("global.md");
    let project_path = temp_dir.path().join("project.md");

    fs::write(&global_path, "# 2026/01/15\n- [ ] Global entry\n").unwrap();
    fs::write(&project_path, "# 2026/01/15\n- [ ] Project entry\n").unwrap();

    storage::reset_journal_context();
    storage::set_journal_context(
        global_path,
        Some(project_path),
        JournalSlot::Project, // Start in project
    );

    let config = Config::default();
    let app = App::new_with_date(config, date).unwrap();

    // Should see project entry
    let has_project = app.entry_indices.iter().any(|&i| {
        if let Line::Entry(e) = &app.lines[i] {
            e.content.contains("Project entry")
        } else {
            false
        }
    });
    assert!(has_project, "Project entry should be visible");

    // Should NOT see global entry
    let has_global = app.entry_indices.iter().any(|&i| {
        if let Line::Entry(e) = &app.lines[i] {
            e.content.contains("Global entry")
        } else {
            false
        }
    });
    assert!(!has_global, "Global entry should not be visible in project");
}
