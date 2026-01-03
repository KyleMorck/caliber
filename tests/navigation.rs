mod helpers;

use chrono::NaiveDate;
use crossterm::event::KeyCode;
use helpers::TestContext;

/// NV-1: Day navigation (h/l)
#[test]
fn test_day_navigation() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] Today entry\n# 2026/01/14\n- [ ] Yesterday entry\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    assert_eq!(ctx.app.current_date, date, "Should start on specified date");
    assert!(
        ctx.screen_contains("Today entry"),
        "Today's entry should be visible"
    );

    // Go back
    ctx.press(KeyCode::Char('h'));
    assert_eq!(
        ctx.app.current_date,
        NaiveDate::from_ymd_opt(2026, 1, 14).unwrap(),
        "Should be on previous day"
    );
    assert!(
        ctx.screen_contains("Yesterday entry"),
        "Yesterday's entry should be visible"
    );

    // Go forward
    ctx.press(KeyCode::Char('l'));
    assert_eq!(ctx.app.current_date, date, "Should be back to today");
    assert!(
        ctx.screen_contains("Today entry"),
        "Today's entry should be visible again"
    );
}

/// NV-2: Jump to today with 't'
#[test]
fn test_jump_to_today() {
    // 't' jumps to actual today (Local::now()), not the app's initial date
    let actual_today = chrono::Local::now().date_naive();
    let past_date = actual_today - chrono::Days::new(5);
    let mut ctx = TestContext::with_date(past_date);

    // Navigate further back
    ctx.press(KeyCode::Char('h'));
    ctx.press(KeyCode::Char('h'));
    assert_eq!(
        ctx.app.current_date,
        past_date - chrono::Days::new(2),
        "Should be 2 more days back"
    );

    // Jump to today
    ctx.press(KeyCode::Char('t'));
    assert_eq!(
        ctx.app.current_date, actual_today,
        "Should jump to actual today"
    );
}

/// NV-3: Entry navigation (j/k/g/G)
#[test]
fn test_entry_navigation() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content =
        "# 2026/01/15\n- [ ] Entry 1\n- [ ] Entry 2\n- [ ] Entry 3\n- [ ] Entry 4\n- [ ] Entry 5\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    // Jump to first
    ctx.press(KeyCode::Char('g'));
    let lines = ctx.render_daily();
    assert!(
        lines.iter().any(|l| l.starts_with("→") && l.contains("Entry 1")),
        "First entry should be selected"
    );

    // Jump to last
    ctx.press(KeyCode::Char('G'));
    let lines = ctx.render_daily();
    assert!(
        lines.iter().any(|l| l.starts_with("→") && l.contains("Entry 5")),
        "Last entry should be selected"
    );

    // Move up
    ctx.press(KeyCode::Char('k'));
    let lines = ctx.render_daily();
    assert!(
        lines.iter().any(|l| l.starts_with("→") && l.contains("Entry 4")),
        "Entry 4 should be selected after k"
    );

    // Move down
    ctx.press(KeyCode::Char('j'));
    let lines = ctx.render_daily();
    assert!(
        lines.iter().any(|l| l.starts_with("→") && l.contains("Entry 5")),
        "Entry 5 should be selected after j"
    );
}

/// NV-4: Navigation with hidden completed
#[test]
fn test_navigation_with_hidden() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] A\n- [x] B\n- [ ] C\n- [x] D\n- [ ] E\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    // Hide completed
    ctx.press(KeyCode::Char('z'));

    // Navigate - should skip hidden entries
    ctx.press(KeyCode::Char('g')); // First visible (A)
    let lines = ctx.render_daily();
    assert!(
        lines.iter().any(|l| l.starts_with("→") && l.contains(" A")),
        "A should be selected"
    );

    ctx.press(KeyCode::Char('j')); // Should go to C (skipping B)
    let lines = ctx.render_daily();
    assert!(
        lines.iter().any(|l| l.starts_with("→") && l.contains(" C")),
        "C should be selected (skipping hidden B)"
    );

    ctx.press(KeyCode::Char('j')); // Should go to E (skipping D)
    let lines = ctx.render_daily();
    assert!(
        lines.iter().any(|l| l.starts_with("→") && l.contains(" E")),
        "E should be selected (skipping hidden D)"
    );
}

/// NV-1: Navigate to future dates
#[test]
fn test_navigate_future() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let mut ctx = TestContext::with_date(date);

    // Navigate to future
    ctx.press(KeyCode::Char('l'));
    assert_eq!(
        ctx.app.current_date,
        NaiveDate::from_ymd_opt(2026, 1, 16).unwrap(),
        "Should be able to navigate to future dates"
    );
}

/// Test bracket keys for day navigation (alternative to h/l)
#[test]
fn test_bracket_key_navigation() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let mut ctx = TestContext::with_date(date);

    ctx.press(KeyCode::Char('['));
    assert_eq!(
        ctx.app.current_date,
        NaiveDate::from_ymd_opt(2026, 1, 14).unwrap(),
        "[ should go to previous day"
    );

    ctx.press(KeyCode::Char(']'));
    assert_eq!(
        ctx.app.current_date,
        NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        "] should go to next day"
    );
}
