mod helpers;

use crossterm::event::KeyCode;
use helpers::TestContext;

/// HI-1: Command hint completion workflow
/// Type partial command, accept hint, verify buffer contains completed command
#[test]
fn test_command_hint_completion() {
    let mut ctx = TestContext::new();

    ctx.press(KeyCode::Char(':'));
    ctx.type_str("qu");
    ctx.press(KeyCode::Right);

    assert_eq!(ctx.app.command_buffer.content(), "quit");
}

/// HI-2: Tag hint completion workflow
/// Create entry with tag, start new entry, complete tag, save, verify persisted
#[test]
fn test_tag_hint_completion() {
    let content = "# 2026/01/15\n- [ ] Task with #feature tag\n";
    let date = chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let mut ctx = TestContext::with_journal_content(date, content);

    ctx.press(KeyCode::Enter);
    ctx.type_str("New task #fe");
    ctx.press(KeyCode::Right);
    ctx.press(KeyCode::Enter);

    assert!(ctx.screen_contains("New task #feature"));
    assert!(ctx.read_journal().contains("New task #feature"));
}

/// HI-3: Filter type hint completion workflow
/// Complete filter syntax and verify filter executes
#[test]
fn test_filter_type_hint_completion() {
    let content = "# 2026/01/15\n- [ ] Incomplete task\n- [x] Completed task\n- A note\n";
    let date = chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let mut ctx = TestContext::with_journal_content(date, content);

    ctx.press(KeyCode::Char('/'));
    ctx.type_str("!ta");
    ctx.press(KeyCode::Right);
    ctx.press(KeyCode::Enter);

    assert!(ctx.screen_contains("Incomplete task"));
    assert!(!ctx.screen_contains("A note"));
}

/// HI-4: Date operation hint completion
#[test]
fn test_date_op_hint_completion() {
    let mut ctx = TestContext::new();

    ctx.press(KeyCode::Char('/'));
    ctx.type_str("@be");
    ctx.press(KeyCode::Right);

    // In QueryInput mode from Daily view, buffer is command_buffer
    assert_eq!(ctx.app.command_buffer.content(), "@before:");
}

/// HI-5: Negation hint completion
#[test]
fn test_negation_hint_completion() {
    let mut ctx = TestContext::new();

    ctx.press(KeyCode::Char('/'));
    ctx.type_str("not:");
    ctx.press(KeyCode::Right);

    let query = ctx.app.command_buffer.content();
    assert!(query.starts_with("not:#"));
}

/// HI-6: Tag hints work with multi-word input
#[test]
fn test_tag_hints_in_multiword_context() {
    let content = "# 2026/01/15\n- [ ] Task with #work tag\n";
    let date = chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let mut ctx = TestContext::with_journal_content(date, content);

    ctx.press(KeyCode::Enter);
    ctx.type_str("Meeting notes #wo");
    ctx.press(KeyCode::Right);
    ctx.press(KeyCode::Enter);

    assert!(ctx.screen_contains("Meeting notes #work"));
}

/// HI-7: Hints dismiss on exact match (no completion available)
#[test]
fn test_exact_match_no_completion() {
    let content = "# 2026/01/15\n- [ ] Task with #bug tag\n";
    let date = chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let mut ctx = TestContext::with_journal_content(date, content);

    ctx.press(KeyCode::Enter);
    ctx.type_str("#bug");
    // Right arrow should move cursor, not complete (since exact match)
    ctx.press(KeyCode::Right);

    let buffer = ctx
        .app
        .edit_buffer
        .as_ref()
        .map(|b| b.content().to_string());
    assert_eq!(
        buffer,
        Some("#bug".to_string()),
        "Should not add anything on exact match"
    );
}

/// HI-8: Escape clears command mode and hints
#[test]
fn test_escape_clears_command_mode() {
    let mut ctx = TestContext::new();

    ctx.press(KeyCode::Char(':'));
    ctx.type_str("da");
    ctx.press(KeyCode::Esc);

    assert!(ctx.app.command_buffer.is_empty());
    assert!(matches!(
        ctx.app.input_mode,
        caliber::app::InputMode::Normal
    ));
}

/// HI-8b: Escape exits query input immediately (single press, even with content)
#[test]
fn test_escape_exits_query_input() {
    let mut ctx = TestContext::new();

    ctx.press(KeyCode::Char('/'));
    ctx.type_str("!tasks");
    ctx.press(KeyCode::Esc);

    assert!(matches!(
        ctx.app.input_mode,
        caliber::app::InputMode::Normal
    ));
}

/// HI-9: Tags are collected from journal on load
#[test]
fn test_tags_collected_from_journal() {
    let content = "# 2026/01/15\n- [ ] #alpha task\n- [ ] #beta task\n- [ ] #alpha again\n";
    let date = chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let mut ctx = TestContext::with_journal_content(date, content);

    ctx.press(KeyCode::Enter);
    ctx.type_str("#a");
    ctx.press(KeyCode::Right);
    ctx.press(KeyCode::Enter);

    // Should complete to #alpha (first alphabetically)
    assert!(ctx.screen_contains("#alpha"));
}
