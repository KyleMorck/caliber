mod helpers;

use crossterm::event::{KeyCode, KeyModifiers};
use helpers::TestContext;

use caliber::app::InputMode;

/// EM-1: Cursor movement commands (Home/End)
#[test]
fn test_cursor_movement_home_end() {
    let mut ctx = TestContext::new();

    // Create entry with known content
    ctx.press(KeyCode::Enter);
    ctx.type_str("hello world");

    // Test Home - moves cursor to start
    ctx.press(KeyCode::Home);
    ctx.type_str("X");
    ctx.press(KeyCode::Enter);

    assert!(
        ctx.screen_contains("Xhello world"),
        "Home should move cursor to start"
    );

    // Edit again and test End
    ctx.press(KeyCode::Char('i'));
    ctx.press(KeyCode::End);
    ctx.type_str("Y");
    ctx.press(KeyCode::Enter);

    assert!(
        ctx.screen_contains("Xhello worldY"),
        "End should move cursor to end"
    );
}

/// EM-1: Ctrl+A and Ctrl+E for start/end navigation
#[test]
fn test_cursor_movement_ctrl_a_e() {
    let mut ctx = TestContext::new();

    ctx.press(KeyCode::Enter);
    ctx.type_str("test content");

    // Ctrl+A moves to start
    ctx.press_with_modifiers(KeyCode::Char('a'), KeyModifiers::CONTROL);
    ctx.type_str("X");
    ctx.press(KeyCode::Enter);

    assert!(
        ctx.screen_contains("Xtest content"),
        "Ctrl+A should move cursor to start"
    );

    // Ctrl+E moves to end
    ctx.press(KeyCode::Char('i'));
    ctx.press_with_modifiers(KeyCode::Char('e'), KeyModifiers::CONTROL);
    ctx.type_str("Y");
    ctx.press(KeyCode::Enter);

    assert!(
        ctx.screen_contains("Xtest contentY"),
        "Ctrl+E should move cursor to end"
    );
}

/// EM-2: Ctrl+W deletes word before cursor
#[test]
fn test_delete_word_back() {
    let mut ctx = TestContext::new();

    ctx.press(KeyCode::Enter);
    ctx.type_str("hello beautiful world");

    // Ctrl+W: delete word before cursor (deletes "world")
    ctx.press_with_modifiers(KeyCode::Char('w'), KeyModifiers::CONTROL);
    ctx.press(KeyCode::Enter);

    assert!(
        ctx.screen_contains("hello beautiful"),
        "Ctrl+W should delete word before cursor"
    );
    assert!(
        !ctx.screen_contains("world"),
        "Ctrl+W should have removed 'world'"
    );
}

/// EM-2: Ctrl+U deletes from cursor to start
#[test]
fn test_delete_to_start() {
    let mut ctx = TestContext::new();

    ctx.press(KeyCode::Enter);
    ctx.type_str("hello world");
    ctx.press(KeyCode::Enter);

    // Edit and delete to start
    ctx.press(KeyCode::Char('i'));
    ctx.press(KeyCode::End);
    ctx.press_with_modifiers(KeyCode::Char('u'), KeyModifiers::CONTROL);
    ctx.type_str("new content");
    ctx.press(KeyCode::Enter);

    assert!(
        ctx.screen_contains("new content"),
        "Ctrl+U should delete to start and allow new content"
    );
    assert!(
        !ctx.screen_contains("hello"),
        "Old content should be deleted"
    );
}

/// EM-3: Tab to commit and add new entry
#[test]
fn test_tab_to_continue() {
    let mut ctx = TestContext::new();

    ctx.press(KeyCode::Enter);
    ctx.type_str("First entry");
    ctx.press(KeyCode::Tab); // Save and create new

    // Should be editing new entry
    assert!(
        matches!(ctx.app.input_mode, InputMode::Edit(_)),
        "Should be in edit mode for new entry"
    );

    ctx.type_str("Second entry");
    ctx.press(KeyCode::Enter);

    // Both entries should exist
    assert!(
        ctx.screen_contains("First entry"),
        "First entry should exist"
    );
    assert!(
        ctx.screen_contains("Second entry"),
        "Second entry should exist"
    );
}

/// EM-4: Cancel edit with Esc restores original content
#[test]
fn test_cancel_edit() {
    let mut ctx = TestContext::new();

    // Create initial entry
    ctx.press(KeyCode::Enter);
    ctx.type_str("Original content");
    ctx.press(KeyCode::Enter);

    // Edit and modify
    ctx.press(KeyCode::Char('i'));
    ctx.press_with_modifiers(KeyCode::Char('u'), KeyModifiers::CONTROL); // Delete all
    ctx.type_str("Modified content");
    ctx.press(KeyCode::Esc); // Cancel

    // Original should be preserved
    assert!(
        ctx.screen_contains("Original content"),
        "Original content should be preserved after cancel"
    );
    assert!(
        !ctx.screen_contains("Modified content"),
        "Modified content should not appear"
    );
}

/// EM-5: Entry type cycling with Shift+Tab (BackTab)
#[test]
fn test_entry_type_cycling() {
    let mut ctx = TestContext::new();

    ctx.press(KeyCode::Enter); // New task (starts as task)
    ctx.type_str("Test entry");

    // Cycle through types: task -> note -> event -> task
    ctx.press(KeyCode::BackTab); // Switch to note
    ctx.press(KeyCode::BackTab); // Switch to event
    ctx.press(KeyCode::BackTab); // Switch back to task
    ctx.press(KeyCode::Enter);

    // Should be a task (shows checkbox)
    assert!(
        ctx.screen_contains("[ ]"),
        "Entry should be a task after cycling back"
    );
}

/// EM-5: Verify note type (no checkbox)
#[test]
fn test_entry_type_note() {
    let mut ctx = TestContext::new();

    ctx.press(KeyCode::Enter);
    ctx.type_str("Note entry");
    ctx.press(KeyCode::BackTab); // Switch to note
    ctx.press(KeyCode::Enter);

    // Note should not have checkbox, just "- "
    assert!(
        ctx.screen_contains("Note entry"),
        "Note content should appear"
    );
    // A note line should not contain "[ ]"
    let line = ctx.find_line("Note entry");
    assert!(
        line.map_or(true, |l| !l.contains("[ ]")),
        "Note should not have checkbox"
    );
}

/// EM-6: Empty entry gets deleted on save
#[test]
fn test_empty_entry_deleted() {
    let mut ctx = TestContext::new();

    // Count initial entries
    let initial_count = ctx.app.entry_indices.len();

    ctx.press(KeyCode::Enter); // New entry
    ctx.press(KeyCode::Enter); // Save empty

    // Entry count should be same (empty entry auto-deleted)
    assert_eq!(
        ctx.app.entry_indices.len(),
        initial_count,
        "Empty entry should be auto-deleted"
    );
}

/// Test basic text input and backspace
#[test]
fn test_basic_text_input() {
    let mut ctx = TestContext::new();

    ctx.press(KeyCode::Enter);
    ctx.type_str("hello");
    ctx.press(KeyCode::Backspace);
    ctx.press(KeyCode::Backspace);
    ctx.type_str("p!");
    ctx.press(KeyCode::Enter);

    assert!(
        ctx.screen_contains("help!"),
        "Backspace should delete characters"
    );
}
