mod helpers;

use chrono::NaiveDate;
use crossterm::event::{KeyCode, KeyModifiers};
use helpers::TestContext;

#[test]
fn later_entry_appears_on_target_date() {
    let source_date = NaiveDate::from_ymd_opt(2026, 1, 10).unwrap();
    let content = "# 2026/01/10\n- [ ] Review doc @01/15\n";
    let mut ctx = TestContext::with_journal_content(source_date, content);

    for _ in 0..5 {
        ctx.press(KeyCode::Char('l'));
    }

    assert!(ctx.screen_contains("Review doc @01/15"));
    assert!(ctx.screen_contains("(01/10)"));
}

#[test]
fn edit_later_entry_updates_source() {
    let view_date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/10\n- [ ] Original @01/15\n";
    let mut ctx = TestContext::with_journal_content(view_date, content);

    assert!(ctx.screen_contains("Original @01/15"));

    // Edit the Later entry
    ctx.press(KeyCode::Char('i'));
    // Clear existing text with Ctrl+U and type new content
    ctx.press_with_modifiers(KeyCode::Char('u'), KeyModifiers::CONTROL);
    ctx.type_str("Modified @01/15");
    ctx.press(KeyCode::Enter);

    let journal = ctx.read_journal();
    assert!(
        journal.contains("Modified @01/15"),
        "Journal should contain modified content. Got: {journal}"
    );
    assert!(
        !journal.contains("Original"),
        "Journal should not contain original content. Got: {journal}"
    );
}

#[test]
fn toggle_completes_later_entry_in_source() {
    let view_date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/10\n- [ ] Later task @01/15\n";
    let mut ctx = TestContext::with_journal_content(view_date, content);

    ctx.press(KeyCode::Char('c'));

    let journal = ctx.read_journal();
    assert!(journal.contains("- [x] Later task @01/15"));
}

#[test]
fn delete_later_entry_removes_from_source() {
    let view_date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/10\n- [ ] Delete me @01/15\n- [ ] Keep me\n";
    let mut ctx = TestContext::with_journal_content(view_date, content);

    // Delete the Later entry
    ctx.press(KeyCode::Char('d'));

    let journal = ctx.read_journal();
    assert!(!journal.contains("Delete me"));
    assert!(journal.contains("Keep me"));
}

#[test]
fn edit_blocked_on_recurring_entry_with_hint() {
    // Create a recurring entry that will project to a Monday
    let monday = NaiveDate::from_ymd_opt(2026, 1, 12).unwrap(); // A Monday
    let content = "# 2026/01/05\n- [ ] Weekly standup @every-monday\n";
    let mut ctx = TestContext::with_journal_content(monday, content);

    assert!(ctx.screen_contains("Weekly standup"));

    ctx.press(KeyCode::Char('i'));

    assert!(ctx.status_contains("Press o to go to source"));

    let journal = ctx.read_journal();
    assert!(journal.contains("Weekly standup @every-monday"));
}

#[test]
fn delete_blocked_on_recurring_entry_with_hint() {
    // Create a recurring entry that will project to a Monday
    let monday = NaiveDate::from_ymd_opt(2026, 1, 12).unwrap(); // A Monday
    let content = "# 2026/01/05\n- [ ] Weekly standup @every-monday\n- [ ] Regular task\n";
    let mut ctx = TestContext::with_journal_content(monday, content);

    ctx.press(KeyCode::Char('d'));

    assert!(ctx.status_contains("Press o to go to source"));

    let journal = ctx.read_journal();
    assert!(journal.contains("Weekly standup"));
    assert!(journal.contains("Regular task"));
}

#[test]
fn natural_date_converts_on_save() {
    let today = chrono::Local::now().date_naive();
    let tomorrow = today + chrono::Days::new(1);
    let expected_date = tomorrow.format("@%m/%d").to_string();

    let mut ctx = TestContext::with_date(today);

    ctx.press(KeyCode::Enter);
    ctx.type_str("Call Bob @tomorrow");
    ctx.press(KeyCode::Enter);

    let journal = ctx.read_journal();
    assert!(
        journal.contains(&expected_date),
        "Natural date @tomorrow should convert to {}",
        expected_date
    );
}
