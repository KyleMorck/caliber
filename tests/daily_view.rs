mod helpers;

use chrono::NaiveDate;
use crossterm::event::KeyCode;
use helpers::TestContext;

/// DV-1: Entry creation positions (o adds below current)
#[test]
fn test_entry_creation_below() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] First\n- [ ] Second\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    // Jump to first entry
    ctx.press(KeyCode::Char('g'));

    // 'o' adds below current
    ctx.press(KeyCode::Char('o'));
    ctx.type_str("Below first");
    ctx.press(KeyCode::Enter);

    // Check order in journal
    let journal = ctx.read_journal();
    let first_pos = journal.find("First").unwrap();
    let below_pos = journal.find("Below first").unwrap();
    let second_pos = journal.find("Second").unwrap();

    assert!(
        first_pos < below_pos && below_pos < second_pos,
        "Entry added with 'o' should be between First and Second"
    );
}

/// DV-1: Entry creation with O (above current)
#[test]
fn test_entry_creation_above() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] First\n- [ ] Second\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    // Jump to second entry
    ctx.press(KeyCode::Char('G'));

    // 'O' adds above current
    ctx.press(KeyCode::Char('O'));
    ctx.type_str("Above second");
    ctx.press(KeyCode::Enter);

    // Check order
    let journal = ctx.read_journal();
    let first_pos = journal.find("First").unwrap();
    let above_pos = journal.find("Above second").unwrap();
    let second_pos = journal.find("Second").unwrap();

    assert!(
        first_pos < above_pos && above_pos < second_pos,
        "Entry added with 'O' should be between First and Second"
    );
}

/// DV-3: Delete and undo (with selection sync verification - catches Bug 4)
#[test]
fn test_delete_and_undo() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] Entry A\n- [ ] Entry B\n- [ ] Entry C\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    // Go to first, then move to middle entry (Entry B)
    ctx.press(KeyCode::Char('g')); // Go to first (Entry A)
    assert_eq!(ctx.selected_index(), 0, "Should be at Entry A");
    ctx.press(KeyCode::Char('j')); // Move to Entry B
    assert_eq!(ctx.selected_index(), 1, "Should be at Entry B");

    // Delete
    ctx.press(KeyCode::Char('d'));
    assert!(!ctx.screen_contains("Entry B"), "Entry B should be deleted");
    assert!(ctx.screen_contains("Entry A"), "Entry A should remain");
    assert!(ctx.screen_contains("Entry C"), "Entry C should remain");

    // Verify selection is still valid after delete
    assert!(
        ctx.selected_index() < ctx.entry_count(),
        "Selection should be valid after delete"
    );
    // After deleting middle entry, selection should stay at index 1 (now Entry C)
    assert_eq!(
        ctx.selected_index(),
        1,
        "Selection should move to next entry after delete"
    );

    // Undo
    ctx.press(KeyCode::Char('u'));
    assert!(ctx.screen_contains("Entry B"), "Entry B should be restored");
}

/// DV-4: Reorder mode
#[test]
fn test_reorder_mode() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] A\n- [ ] B\n- [ ] C\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    // Select A, enter reorder
    ctx.press(KeyCode::Char('g'));
    ctx.press(KeyCode::Char('r'));

    // Move A down
    ctx.press(KeyCode::Char('j'));

    // Confirm
    ctx.press(KeyCode::Enter);

    // Verify order in journal: B, A, C
    let journal = ctx.read_journal();
    let b_pos = journal.find(" B").unwrap();
    let a_pos = journal.find(" A").unwrap();
    let c_pos = journal.find(" C").unwrap();
    assert!(
        b_pos < a_pos && a_pos < c_pos,
        "Order should be B, A, C after reorder"
    );
}

/// DV-5: Reorder cancel
#[test]
fn test_reorder_cancel() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] A\n- [ ] B\n- [ ] C\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    // Select A, enter reorder
    ctx.press(KeyCode::Char('g'));
    ctx.press(KeyCode::Char('r'));

    // Move A down
    ctx.press(KeyCode::Char('j'));

    // Cancel
    ctx.press(KeyCode::Esc);

    // Verify original order in journal: A, B, C
    let journal = ctx.read_journal();
    let a_pos = journal.find(" A").unwrap();
    let b_pos = journal.find(" B").unwrap();
    let c_pos = journal.find(" C").unwrap();
    assert!(
        a_pos < b_pos && b_pos < c_pos,
        "Order should be A, B, C after cancel"
    );
}

/// DV-6: Hide completed toggle
#[test]
fn test_hide_completed() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] Incomplete\n- [x] Completed\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    // Both visible initially
    assert!(
        ctx.screen_contains("Incomplete"),
        "Incomplete should be visible initially"
    );
    assert!(
        ctx.screen_contains("Completed"),
        "Completed should be visible initially"
    );

    // Toggle hide
    ctx.press(KeyCode::Char('z'));
    assert!(
        ctx.screen_contains("Incomplete"),
        "Incomplete should remain visible"
    );
    assert!(
        !ctx.screen_contains("Completed"),
        "Completed should be hidden"
    );

    // Toggle back
    ctx.press(KeyCode::Char('z'));
    assert!(
        ctx.screen_contains("Completed"),
        "Completed should be visible again"
    );
}

/// DV-7: Sort entries
#[test]
fn test_sort_entries() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    // Mixed order: incomplete, note, completed
    // Default sort order: completed, events, notes, uncompleted
    let content = "# 2026/01/15\n- [ ] Incomplete task\n- A note\n- [x] Completed task\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    // Sort
    ctx.press(KeyCode::Char('s'));

    // Verify sorted per default: completed, notes, incomplete
    let journal = ctx.read_journal();
    let completed_pos = journal.find("Completed task").unwrap();
    let note_pos = journal.find("A note").unwrap();
    let incomplete_pos = journal.find("Incomplete task").unwrap();
    assert!(
        completed_pos < note_pos && note_pos < incomplete_pos,
        "Default sort: completed tasks, then notes, then incomplete tasks"
    );
}

/// CP-4: Toggle task completion
#[test]
fn test_toggle_completion() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] My task\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    // Toggle to complete
    ctx.press(KeyCode::Char('c'));
    assert!(
        ctx.screen_contains("[x]"),
        "Task should show completed marker"
    );

    // Toggle back
    ctx.press(KeyCode::Char('c'));
    assert!(
        ctx.screen_contains("[ ]"),
        "Task should show incomplete marker"
    );

    // Verify persistence of final state
    let journal = ctx.read_journal();
    assert!(
        journal.contains("- [ ] My task"),
        "Final state should be incomplete"
    );
}

/// Test selection sync after deleting last entry
#[test]
fn test_selection_after_delete_last() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] A\n- [ ] B\n- [ ] C\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    ctx.press(KeyCode::Char('G')); // Go to last (C)
    assert_eq!(ctx.selected_index(), 2, "Should be at last entry");

    ctx.press(KeyCode::Char('d')); // Delete C
    assert_eq!(
        ctx.selected_index(),
        1,
        "Selection should move to new last entry"
    );
    assert!(ctx.screen_contains("B"), "Entry B should now be last");
    assert!(!ctx.screen_contains(" C"), "Entry C should be deleted");
}

/// Test selection sync after deleting middle entry
#[test]
fn test_selection_after_delete_middle() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] A\n- [ ] B\n- [ ] C\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    ctx.press(KeyCode::Char('g')); // Go to first
    ctx.press(KeyCode::Char('j')); // Move to B
    assert_eq!(ctx.selected_index(), 1, "Should be at middle entry B");

    ctx.press(KeyCode::Char('d')); // Delete B
    assert!(
        ctx.selected_index() < ctx.entry_count(),
        "Selection must be within valid range"
    );
    // Selection should be on C (now at index 1 in remaining [A, C])
    assert_eq!(
        ctx.selected_index(),
        1,
        "Selection should stay at same index (now C)"
    );
}

/// DV-8: Scroll behavior with many entries
#[test]
fn test_scroll_behavior() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    // Create 30 entries (more than fit on typical screen)
    let mut content = "# 2026/01/15\n".to_string();
    for i in 1..=30 {
        content.push_str(&format!("- [ ] Entry {}\n", i));
    }
    let mut ctx = TestContext::with_journal_content(date, &content);

    // Jump to last entry
    ctx.press(KeyCode::Char('G'));
    assert_eq!(
        ctx.selected_index(),
        29,
        "Should be at last entry (index 29)"
    );
    assert!(
        ctx.screen_contains("Entry 30"),
        "Last entry should be visible"
    );

    // Navigate up one at a time
    ctx.press(KeyCode::Char('k'));
    assert_eq!(ctx.selected_index(), 28, "Should be at entry 29");

    // Jump to top
    ctx.press(KeyCode::Char('g'));
    assert_eq!(ctx.selected_index(), 0, "Should be at first entry");
    assert!(
        ctx.screen_contains("Entry 1"),
        "First entry should be visible"
    );
}

/// DV-9: Yank entry
#[test]
fn test_yank_entry() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] Yank this content\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    // Yank the entry
    ctx.press(KeyCode::Char('y'));

    // Verify status message appears (the app should show "Yanked" or similar)
    // Note: The actual clipboard operation can't be tested, but we verify the action completes
    // and the entry is still there
    assert!(
        ctx.screen_contains("Yank this content"),
        "Entry should still be visible after yank"
    );
}
