mod helpers;

use chrono::NaiveDate;
use crossterm::event::KeyCode;
use helpers::TestContext;

/// RE-1: Entry with @every-day appears on all days
#[test]
fn test_recurring_daily_appears_every_day() {
    let source_date = NaiveDate::from_ymd_opt(2026, 1, 10).unwrap();
    let content = "# 2026/01/10\n- [ ] Stand-up meeting @every-day\n";
    let mut ctx = TestContext::with_journal_content(source_date, content);

    // Navigate forward 3 days
    for _ in 0..3 {
        ctx.press(KeyCode::Char('l'));
    }

    // Entry should appear as recurring entry
    assert!(
        ctx.screen_contains("Stand-up meeting"),
        "Recurring entry should appear on target date"
    );
    // Should show source date indicator
    assert!(
        ctx.screen_contains("(01/10)"),
        "Source date indicator should appear"
    );
}

/// RE-2: Entry with @every-weekday only appears on weekdays
#[test]
fn test_recurring_weekday_appears_on_weekdays() {
    // Start on Monday 2026-01-12
    let monday = NaiveDate::from_ymd_opt(2026, 1, 12).unwrap();
    let content = "# 2026/01/01\n- [ ] Weekday task @every-weekday\n";
    let mut ctx = TestContext::with_journal_content(monday, content);

    // Should appear on Monday
    assert!(
        ctx.screen_contains("Weekday task"),
        "Recurring entry should appear on Monday"
    );

    // Navigate to Saturday (5 days forward)
    for _ in 0..5 {
        ctx.press(KeyCode::Char('l'));
    }

    // Should NOT appear on Saturday
    assert!(
        !ctx.screen_contains("Weekday task"),
        "Recurring entry should not appear on Saturday"
    );
}

/// RE-3: Entry with @every-monday only appears on Mondays
#[test]
fn test_recurring_weekly_appears_on_day() {
    // Start on Monday 2026-01-12
    let monday = NaiveDate::from_ymd_opt(2026, 1, 12).unwrap();
    let content = "# 2026/01/01\n- [ ] Weekly review @every-mon\n";
    let mut ctx = TestContext::with_journal_content(monday, content);

    // Should appear on Monday
    assert!(
        ctx.screen_contains("Weekly review"),
        "Recurring entry should appear on Monday"
    );

    // Navigate to Tuesday (1 day forward)
    ctx.press(KeyCode::Char('l'));

    // Should NOT appear on Tuesday
    assert!(
        !ctx.screen_contains("Weekly review"),
        "Recurring entry should not appear on Tuesday"
    );
}

/// RE-4: Toggle on recurring entry materializes completed copy
#[test]
fn test_toggle_recurring_materializes_completed() {
    // We're viewing today with a recurring entry from the past
    let today = chrono::Local::now().date_naive();
    let past = today - chrono::Days::new(7);
    let content = format!(
        "# {}\n- [ ] Daily task @every-day\n",
        past.format("%Y/%m/%d")
    );
    let mut ctx = TestContext::with_journal_content(today, &content);

    // Toggle the recurring entry
    ctx.press(KeyCode::Char('c'));

    // Check journal - should have materialized completed entry on today
    let journal = ctx.read_journal();
    let today_section = today.format("# %Y/%m/%d").to_string();

    // Original recurring entry should still exist
    assert!(
        journal.contains("- [ ] Daily task @every-day"),
        "Original recurring entry should remain"
    );

    // A completed copy should be added to today
    assert!(
        journal.contains(&today_section),
        "Today's section should exist"
    );
    assert!(
        journal.contains("- [x] Daily task"),
        "Completed copy should be added to today"
    );
}

/// RE-5: Edit is blocked on recurring entries
#[test]
fn test_edit_blocked_on_recurring() {
    let today = chrono::Local::now().date_naive();
    let past = today - chrono::Days::new(1);
    let content = format!(
        "# {}\n- [ ] Daily task @every-day\n",
        past.format("%Y/%m/%d")
    );
    let mut ctx = TestContext::with_journal_content(today, &content);

    // Try to edit the recurring entry
    ctx.press(KeyCode::Char('i'));

    // Should show status message
    assert!(
        ctx.status_contains("Press o to go to source"),
        "Edit should be blocked with go-to-source hint"
    );
}

/// RE-6: Delete is blocked on recurring entries
#[test]
fn test_delete_blocked_on_recurring() {
    let today = chrono::Local::now().date_naive();
    let past = today - chrono::Days::new(1);
    let content = format!(
        "# {}\n- [ ] Daily task @every-day\n",
        past.format("%Y/%m/%d")
    );
    let mut ctx = TestContext::with_journal_content(today, &content);

    // Try to delete the recurring entry
    ctx.press(KeyCode::Char('d'));

    // Should show status message
    assert!(
        ctx.status_contains("Press o to go to source"),
        "Delete should be blocked with go-to-source hint"
    );

    // Journal should be unchanged
    let journal = ctx.read_journal();
    assert!(
        journal.contains("Daily task @every-day"),
        "Entry should not have been deleted"
    );
}

/// RE-7: @recurring filter shows recurring entries
#[test]
fn test_recurring_filter() {
    let today = chrono::Local::now().date_naive();
    let content = format!(
        "# {}\n- [ ] Daily task @every-day\n- [ ] One-time task @01/15\n- [ ] Regular task\n",
        today.format("%Y/%m/%d")
    );
    let mut ctx = TestContext::with_journal_content(today, &content);

    // Filter for @recurring
    ctx.press(KeyCode::Char('/'));
    ctx.type_str("@recurring");
    ctx.press(KeyCode::Enter);

    // Recurring entry should appear
    assert!(
        ctx.screen_contains("Daily task @every-day"),
        "Recurring entry should appear in @recurring filter"
    );

    // One-time and regular entries should not appear
    assert!(
        !ctx.screen_contains("One-time task"),
        "Later entry should not appear in @recurring filter"
    );
    assert!(
        !ctx.screen_contains("Regular task"),
        "Regular entry should not appear in @recurring filter"
    );
}
