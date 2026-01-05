mod helpers;

use chrono::NaiveDate;
use crossterm::event::KeyCode;
use helpers::TestContext;

use caliber::app::ViewMode;

/// FV-1: Basic tag filter
#[test]
fn test_tag_filter() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content =
        "# 2026/01/15\n- [ ] Task with #work\n- [ ] Task with #personal\n- Note with #work\n";
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

/// FV-3: Completed task filter (!tasks vs !completed)
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

    // Exit and try !completed
    ctx.press(KeyCode::Tab);
    ctx.press(KeyCode::Char('/'));
    ctx.type_str("!completed");
    ctx.press(KeyCode::Enter);
    assert!(
        !ctx.screen_contains("Incomplete task"),
        "Incomplete should not appear with !completed"
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
    let content =
        "# 2026/01/15\n- [ ] Work task #work\n- [ ] Personal task #personal\n- Work note #work\n";
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
    ctx.type_str("!tasks");
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
    ctx.press(KeyCode::Char('d'));

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

/// FV-5: Negation filter (not:#tag)
#[test]
fn test_negation_filter() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content =
        "# 2026/01/15\n- [ ] Work task #work\n- [ ] Personal task #personal\n- [ ] Untagged task\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    ctx.press(KeyCode::Char('/'));
    ctx.type_str("not:#work");
    ctx.press(KeyCode::Enter);

    // Check entry content, not just tags (since the query itself may show "#work")
    assert!(
        !ctx.screen_contains("Work task"),
        "Entry with #work should not appear"
    );
    assert!(
        ctx.screen_contains("Personal task"),
        "Entry with #personal should appear"
    );
    assert!(
        ctx.screen_contains("Untagged"),
        "Untagged entry should appear"
    );
}

/// FV-6: Date range filter (@after:DATE)
#[test]
fn test_date_range_filter_after() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] Today entry\n# 2026/01/14\n- [ ] Yesterday entry\n# 2026/01/10\n- [ ] Old entry\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    ctx.press(KeyCode::Char('/'));
    ctx.type_str("@after:1/14/26");
    ctx.press(KeyCode::Enter);

    assert!(
        ctx.screen_contains("Today entry"),
        "Today entry should appear"
    );
    assert!(
        ctx.screen_contains("Yesterday entry"),
        "Yesterday entry should appear"
    );
    assert!(
        !ctx.screen_contains("Old entry"),
        "Old entry should not appear"
    );
}

/// FV-6: Date range filter (@before:DATE)
#[test]
fn test_date_range_filter_before() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] Today entry\n# 2026/01/14\n- [ ] Yesterday entry\n# 2026/01/10\n- [ ] Old entry\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    ctx.press(KeyCode::Char('/'));
    ctx.type_str("@before:1/14/26");
    ctx.press(KeyCode::Enter);

    assert!(
        !ctx.screen_contains("Today entry"),
        "Today entry should not appear"
    );
    assert!(
        ctx.screen_contains("Yesterday entry"),
        "Yesterday entry should appear (inclusive)"
    );
    assert!(ctx.screen_contains("Old entry"), "Old entry should appear");
}

/// FV-5: Negation filter for types (not:!tasks)
#[test]
fn test_negation_type_filter() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] A task\n- A note\n* An event\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    ctx.press(KeyCode::Char('/'));
    ctx.type_str("not:!tasks");
    ctx.press(KeyCode::Enter);

    assert!(
        !ctx.screen_contains("A task"),
        "Tasks should not appear with not:!tasks"
    );
    assert!(ctx.screen_contains("A note"), "Notes should appear");
    assert!(ctx.screen_contains("An event"), "Events should appear");
}

/// FV-11: Filter refresh after external changes
#[test]
fn test_filter_refresh() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] Task without tag\n- [ ] Task with #work\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    // Filter for #work - should find 1 entry
    ctx.press(KeyCode::Char('/'));
    ctx.type_str("#work");
    ctx.press(KeyCode::Enter);

    assert!(
        ctx.screen_contains("Task with #work"),
        "#work task should appear"
    );
    assert!(
        !ctx.screen_contains("Task without tag"),
        "Untagged task should not appear initially"
    );

    // Exit filter, edit the untagged entry to add #work
    ctx.press(KeyCode::Tab);
    ctx.press(KeyCode::Char('g')); // Go to first entry (the untagged one)
    ctx.press(KeyCode::Char('i'));
    ctx.press(KeyCode::End);
    ctx.type_str(" #work");
    ctx.press(KeyCode::Enter);

    // Return to filter and refresh
    ctx.press(KeyCode::Tab);
    ctx.press(KeyCode::Char('r')); // Refresh filter

    // Now both entries should appear
    assert!(
        ctx.screen_contains("Task without tag"),
        "Newly tagged task should appear after refresh"
    );
    assert!(
        ctx.screen_contains("Task with #work"),
        "Original #work task should still appear"
    );
}

/// FV-13: Favorite tag quick filter (number keys)
#[test]
fn test_favorite_tag_quick_filter() {
    use std::collections::HashMap;

    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content =
        "# 2026/01/15\n- [ ] Task with #work\n- [ ] Task with #personal\n- [ ] Task without tags\n";

    // Create config with favorite tag 1 = "work"
    let mut config = caliber::config::Config::default();
    let mut tags = HashMap::new();
    tags.insert("1".to_string(), "work".to_string());
    config.favorite_tags = tags;

    let mut ctx = TestContext::with_config_and_content(date, content, config);

    // Press '1' to quick filter by #work (favorite tag 1)
    ctx.press(KeyCode::Char('1'));

    // Should be in filter mode with #work entries
    assert!(
        matches!(ctx.app.view, ViewMode::Filter(_)),
        "Should be in filter view"
    );
    assert!(
        ctx.screen_contains("Task with #work"),
        "#work task should appear"
    );
    assert!(
        !ctx.screen_contains("Task with #personal"),
        "#personal task should not appear"
    );
    assert!(
        !ctx.screen_contains("Task without tags"),
        "Untagged task should not appear"
    );
}

/// FV-9b: Delete from filter with undo/redo
#[test]
fn test_delete_from_filter_undo_redo() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] Entry A\n- [ ] Entry B\n- [ ] Entry C\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    // Enter filter view
    ctx.press(KeyCode::Char('/'));
    ctx.type_str("Entry");
    ctx.press(KeyCode::Enter);

    // Verify all entries visible
    assert!(ctx.screen_contains("Entry A"), "Entry A should appear");
    assert!(ctx.screen_contains("Entry B"), "Entry B should appear");
    assert!(ctx.screen_contains("Entry C"), "Entry C should appear");

    // Select Entry B (second in list) and delete
    ctx.press(KeyCode::Char('k')); // Move up from C to B
    ctx.press(KeyCode::Char('d'));

    // Verify B is deleted
    assert!(ctx.screen_contains("Entry A"), "Entry A should remain");
    assert!(!ctx.screen_contains("Entry B"), "Entry B should be deleted");
    assert!(ctx.screen_contains("Entry C"), "Entry C should remain");

    // Undo the delete
    ctx.press(KeyCode::Char('u'));

    // Verify B is restored
    assert!(
        ctx.screen_contains("Entry B"),
        "Entry B should be restored after undo"
    );

    // Redo the delete
    ctx.press(KeyCode::Char('U'));

    // Verify B is deleted again
    assert!(
        !ctx.screen_contains("Entry B"),
        "Entry B should be deleted after redo"
    );

    // Undo again and verify persistence
    ctx.press(KeyCode::Char('u'));
    let journal = ctx.read_journal();
    assert!(
        journal.contains("Entry B"),
        "Entry B should be persisted after final undo"
    );
}

/// FV-14: Saved filter expansion ($name syntax)
#[test]
fn test_saved_filter_expansion() {
    use std::collections::HashMap;

    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content =
        "# 2026/01/15\n- [ ] Urgent task #urgent\n- [ ] Normal task #work\n- A note #urgent\n";

    // Create config with saved filter
    let mut config = caliber::config::Config::default();
    let mut filters = HashMap::new();
    filters.insert("t".to_string(), "!tasks".to_string());
    config.filters = filters;

    let mut ctx = TestContext::with_config_and_content(date, content, config);

    // Use saved filter $t (expands to !tasks) combined with #urgent
    ctx.press(KeyCode::Char('/'));
    ctx.type_str("$t #urgent");
    ctx.press(KeyCode::Enter);

    // Should show only incomplete tasks with #urgent (not notes)
    assert!(
        ctx.screen_contains("Urgent task #urgent"),
        "Urgent task should appear"
    );
    assert!(
        !ctx.screen_contains("Normal task"),
        "Non-urgent task should not appear"
    );
    assert!(
        !ctx.screen_contains("A note"),
        "Note should not appear (even though it has #urgent)"
    );
}
