mod helpers;

use chrono::NaiveDate;
use crossterm::event::KeyCode;
use helpers::TestContext;

use caliber::app::ViewMode;

/// FV-1: Basic tag filter
#[test]
fn test_tag_filter() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] Task with #work\n- [ ] Task with #personal\n- Note with #work\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    ctx.press(KeyCode::Char('/'));
    ctx.type_str("#work");
    ctx.press(KeyCode::Enter);

    assert!(
        matches!(ctx.app.view, ViewMode::Filter(_)),
        "Should be in filter view"
    );
    assert!(
        ctx.screen_contains("Task with #work"),
        "Task with #work should appear"
    );
    assert!(
        ctx.screen_contains("Note with #work"),
        "Note with #work should appear"
    );
    assert!(
        !ctx.screen_contains("#personal"),
        "Entry with #personal should not appear"
    );
}

/// FV-2: Type filter (!tasks)
#[test]
fn test_type_filter_tasks() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] A task\n- A note\n* An event\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    // Filter tasks
    ctx.press(KeyCode::Char('/'));
    ctx.type_str("!tasks");
    ctx.press(KeyCode::Enter);

    assert!(ctx.screen_contains("A task"), "Task should appear");
    assert!(!ctx.screen_contains("A note"), "Note should not appear");
    assert!(!ctx.screen_contains("An event"), "Event should not appear");
}

/// FV-2: Type filter (!notes)
#[test]
fn test_type_filter_notes() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] A task\n- A note\n* An event\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    ctx.press(KeyCode::Char('/'));
    ctx.type_str("!notes");
    ctx.press(KeyCode::Enter);

    assert!(!ctx.screen_contains("A task"), "Task should not appear");
    assert!(ctx.screen_contains("A note"), "Note should appear");
    assert!(!ctx.screen_contains("An event"), "Event should not appear");
}

/// FV-3: Completed task filter (!tasks vs !tasks/done)
#[test]
fn test_completed_task_filter() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] Incomplete task\n- [x] Completed task\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    // !tasks shows only incomplete
    ctx.press(KeyCode::Char('/'));
    ctx.type_str("!tasks");
    ctx.press(KeyCode::Enter);
    assert!(
        ctx.screen_contains("Incomplete task"),
        "Incomplete should appear"
    );
    assert!(
        !ctx.screen_contains("Completed task"),
        "Completed should not appear with !tasks"
    );

    // Exit and try !tasks/done
    ctx.press(KeyCode::Tab);
    ctx.press(KeyCode::Char('/'));
    ctx.type_str("!tasks/done");
    ctx.press(KeyCode::Enter);
    assert!(
        !ctx.screen_contains("Incomplete task"),
        "Incomplete should not appear with !tasks/done"
    );
    assert!(
        ctx.screen_contains("Completed task"),
        "Completed should appear"
    );
}

/// FV-4: Combined filters
#[test]
fn test_combined_filters() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] Work task #work\n- [ ] Personal task #personal\n- Work note #work\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    // !tasks #work - only incomplete tasks with #work
    ctx.press(KeyCode::Char('/'));
    ctx.type_str("!tasks #work");
    ctx.press(KeyCode::Enter);

    assert!(
        ctx.screen_contains("Work task #work"),
        "Work task should appear"
    );
    assert!(
        !ctx.screen_contains("Personal task"),
        "Personal task should not appear"
    );
    assert!(
        !ctx.screen_contains("Work note"),
        "Note should not appear (not a task)"
    );
}

/// FV-7: Edit from filter persists changes
#[test]
fn test_edit_from_filter() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] Original task\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    // Filter and edit
    ctx.press(KeyCode::Char('/'));
    ctx.type_str("task");
    ctx.press(KeyCode::Enter);

    ctx.press(KeyCode::Char('i')); // Edit
    ctx.press(KeyCode::End);
    ctx.type_str(" modified");
    ctx.press(KeyCode::Enter);

    // Exit filter and check
    ctx.press(KeyCode::Tab);
    assert!(
        ctx.screen_contains("Original task modified"),
        "Modified content should appear in daily view"
    );

    // Check persistence
    let journal = ctx.read_journal();
    assert!(
        journal.contains("Original task modified"),
        "Change should be persisted"
    );
}

/// FV-8: Toggle completion from filter
#[test]
fn test_toggle_from_filter() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] My task\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    ctx.press(KeyCode::Char('/'));
    ctx.type_str("!tasks/all");
    ctx.press(KeyCode::Enter);

    ctx.press(KeyCode::Char('c')); // Toggle

    // Check persistence
    let journal = ctx.read_journal();
    assert!(
        journal.contains("- [x] My task"),
        "Task should be marked complete"
    );
}

/// FV-10: Quick add from filter
#[test]
fn test_quick_add_from_filter() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let mut ctx = TestContext::with_date(date);

    // Enter filter mode
    ctx.press(KeyCode::Char('/'));
    ctx.type_str("!tasks");
    ctx.press(KeyCode::Enter);

    ctx.press(KeyCode::Enter); // Quick add
    ctx.type_str("New from filter");
    ctx.press(KeyCode::Enter);

    // Verify entry appears in filter results (shows the entry we just added)
    assert!(
        ctx.screen_contains("New from filter"),
        "Entry should appear in filter results after creation"
    );

    // Verify entry was persisted to journal
    let journal = ctx.read_journal();
    assert!(
        journal.contains("New from filter"),
        "Entry should be saved to journal"
    );
}

/// FV-12: Return to last filter with Tab
#[test]
fn test_return_to_filter() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] Task #work\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    // Filter
    ctx.press(KeyCode::Char('/'));
    ctx.type_str("#work");
    ctx.press(KeyCode::Enter);

    // Exit
    ctx.press(KeyCode::Tab);
    assert!(
        matches!(ctx.app.view, ViewMode::Daily(_)),
        "Should be in daily view"
    );

    // Return
    ctx.press(KeyCode::Tab);
    assert!(
        matches!(ctx.app.view, ViewMode::Filter(_)),
        "Should be back in filter view"
    );
    assert!(
        ctx.screen_contains("#work"),
        "Filter results should still show #work entries"
    );
}

/// FV-9: Delete from filter
#[test]
fn test_delete_from_filter() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] Delete me\n- [ ] Keep me\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    // Filter for specific entry
    ctx.press(KeyCode::Char('/'));
    ctx.type_str("Delete");
    ctx.press(KeyCode::Enter);

    // Delete
    ctx.press(KeyCode::Char('x'));

    // Exit and verify
    ctx.press(KeyCode::Tab);
    assert!(
        !ctx.screen_contains("Delete me"),
        "Deleted entry should be gone"
    );
    assert!(ctx.screen_contains("Keep me"), "Other entry should remain");

    // Check persistence
    let journal = ctx.read_journal();
    assert!(
        !journal.contains("Delete me"),
        "Deleted entry should be removed from file"
    );
}
