mod helpers;

use chrono::NaiveDate;
use crossterm::event::KeyCode;
use helpers::TestContext;

/// LE-1: Entry with @date appears in target day's Later section
#[test]
fn test_later_entry_appears_on_target_date() {
    let source_date = NaiveDate::from_ymd_opt(2026, 1, 10).unwrap();
    let content = "# 2026/01/10\n- [ ] Review doc @01/15\n";
    let mut ctx = TestContext::with_journal_content(source_date, content);

    // Navigate to 1/15 (5 days forward)
    for _ in 0..5 {
        ctx.press(KeyCode::Char('l'));
    }

    // Entry should appear as later entry
    assert!(
        ctx.screen_contains("Review doc @01/15"),
        "Later entry should appear on target date"
    );
    // Should show source date indicator
    assert!(
        ctx.screen_contains("(01/10)"),
        "Source date indicator should appear"
    );
}

/// LE-3: Edit later entry updates source
#[test]
fn test_edit_later_entry() {
    // Start viewing 1/15, with entry created on 1/10 targeting 1/15
    let view_date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/10\n- [ ] Original @01/15\n";
    let mut ctx = TestContext::with_journal_content(view_date, content);

    // We're on 1/15, should see later entry from 1/10
    assert!(
        ctx.screen_contains("Original @01/15"),
        "Later entry should be visible"
    );

    // Edit the later entry
    ctx.press(KeyCode::Char('i'));
    ctx.press(KeyCode::End);
    ctx.type_str(" modified");
    ctx.press(KeyCode::Enter);

    // Check journal was updated
    let journal = ctx.read_journal();
    assert!(
        journal.contains("Original @01/15 modified"),
        "Later entry edit should be persisted"
    );
}

/// LE-4: Toggle later entry completion
#[test]
fn test_toggle_later_entry() {
    let view_date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/10\n- [ ] Later task @01/15\n";
    let mut ctx = TestContext::with_journal_content(view_date, content);

    ctx.press(KeyCode::Char('c')); // Toggle completion

    let journal = ctx.read_journal();
    assert!(
        journal.contains("- [x] Later task @01/15"),
        "Later entry should be marked complete"
    );
}

/// LE-5: Delete later entry removes from source
#[test]
fn test_delete_later_entry() {
    let view_date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/10\n- [ ] Delete me @01/15\n- [ ] Keep me\n";
    let mut ctx = TestContext::with_journal_content(view_date, content);

    // Delete the later entry
    ctx.press(KeyCode::Char('d'));

    let journal = ctx.read_journal();
    assert!(
        !journal.contains("Delete me"),
        "Deleted later entry should be removed"
    );
    assert!(journal.contains("Keep me"), "Other entry should remain");
}

/// LE-2: Natural date conversion (@tomorrow -> @MM/DD)
#[test]
fn test_natural_date_conversion() {
    // Use the actual current date for this test since natural date conversion
    // uses Local::now(), not the app's current_date
    let today = chrono::Local::now().date_naive();
    let tomorrow = today + chrono::Days::new(1);
    let expected_date = tomorrow.format("@%m/%d").to_string();

    let mut ctx = TestContext::with_date(today);

    // Create entry with natural date
    ctx.press(KeyCode::Enter);
    ctx.type_str("Call Bob @tomorrow");
    ctx.press(KeyCode::Enter);

    // Check journal - should have converted @tomorrow to tomorrow's actual date
    let journal = ctx.read_journal();
    assert!(
        journal.contains(&expected_date),
        "Natural date @tomorrow should convert to {}",
        expected_date
    );
}

/// LE-7: Overdue filter shows entries with past @dates
#[test]
fn test_overdue_filter() {
    // Create entries with past dates using actual today for proper filtering
    let today = chrono::Local::now().date_naive();
    let yesterday = today - chrono::Days::new(1);
    // Use MM/DD format for past date (will prefer past interpretation)
    let past_date = yesterday.format("@%m/%d").to_string();
    // Use explicit year for future date to avoid being interpreted as last year
    let future_date = (today + chrono::Days::new(5))
        .format("@%m/%d/%y")
        .to_string();

    // Create journal with entries that have past and future @dates
    let content = format!(
        "# {}\n- [ ] Past due task {}\n- [ ] Future task {}\n- [ ] No date task\n",
        today.format("%Y/%m/%d"),
        past_date,
        future_date
    );
    let mut ctx = TestContext::with_journal_content(today, &content);

    // Filter for @overdue
    ctx.press(KeyCode::Char('/'));
    ctx.type_str("@overdue");
    ctx.press(KeyCode::Enter);

    // Past due entry should appear
    assert!(
        ctx.screen_contains("Past due task"),
        "Overdue entry should appear in @overdue filter"
    );

    // Future and undated entries should not appear
    assert!(
        !ctx.screen_contains("Future task"),
        "Future entry should not appear in @overdue filter"
    );
    assert!(
        !ctx.screen_contains("No date task"),
        "Undated entry should not appear in @overdue filter"
    );
}
