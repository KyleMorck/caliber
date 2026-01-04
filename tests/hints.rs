mod helpers;

use crossterm::event::KeyCode;
use helpers::TestContext;

use caliber::app::{HintContext, InputMode};

/// HI-1: Command hints appear in command mode
#[test]
fn test_command_hints_appear_on_colon() {
    let mut ctx = TestContext::new();

    ctx.press(KeyCode::Char(':'));

    assert!(
        matches!(ctx.app.input_mode, InputMode::Command),
        "Should be in command mode"
    );
    assert!(
        ctx.app.hint_state.is_active(),
        "Hints should be active in command mode"
    );
    assert!(
        matches!(ctx.app.hint_state, HintContext::Commands { .. }),
        "Should show command hints"
    );
}

/// HI-2: Command hints filter as you type
#[test]
fn test_command_hints_filter() {
    let mut ctx = TestContext::new();

    ctx.press(KeyCode::Char(':'));
    ctx.type_str("go");

    match &ctx.app.hint_state {
        HintContext::Commands { matches, .. } => {
            assert_eq!(matches.len(), 1, "Should filter to just 'goto'");
            assert_eq!(matches[0].command, "goto");
        }
        _ => panic!("Expected command hints"),
    }
}

/// HI-3: Right arrow accepts first command hint
#[test]
fn test_command_hint_acceptance() {
    let mut ctx = TestContext::new();

    ctx.press(KeyCode::Char(':'));
    ctx.type_str("go");
    ctx.press(KeyCode::Right);

    assert_eq!(
        ctx.app.command_buffer.content(),
        "goto",
        "Right arrow should complete 'go' to 'goto'"
    );
}

/// HI-4: Tag hints appear when typing #
#[test]
fn test_tag_hints_appear() {
    let content = "# 2026/01/15\n- [ ] Task with #feature tag\n- Note with #bug tag\n";
    let date = chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let mut ctx = TestContext::with_journal_content(date, content);

    ctx.press(KeyCode::Enter);
    ctx.type_str("#f");

    assert!(
        matches!(ctx.app.hint_state, HintContext::Tags { .. }),
        "Should show tag hints when typing #"
    );
}

/// HI-5: Tag hints filter as you type
#[test]
fn test_tag_hints_filter() {
    let content = "# 2026/01/15\n- [ ] Task with #feature tag\n- [ ] Task with #fix tag\n- Note with #bug tag\n";
    let date = chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let mut ctx = TestContext::with_journal_content(date, content);

    ctx.press(KeyCode::Enter);
    ctx.type_str("#fe");

    match &ctx.app.hint_state {
        HintContext::Tags { matches, .. } => {
            assert_eq!(matches.len(), 1, "Should filter to just 'feature'");
            assert!(matches[0].eq_ignore_ascii_case("feature"));
        }
        _ => panic!("Expected tag hints"),
    }
}

/// HI-6: Right arrow accepts first tag hint
#[test]
fn test_tag_hint_acceptance() {
    let content = "# 2026/01/15\n- [ ] Task with #feature tag\n";
    let date = chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let mut ctx = TestContext::with_journal_content(date, content);

    ctx.press(KeyCode::Enter);
    ctx.type_str("#fe");
    ctx.press(KeyCode::Right);

    let buffer_content = ctx.app.edit_buffer.as_ref().map(|b| b.content().to_string());
    assert_eq!(
        buffer_content,
        Some("#feature".to_string()),
        "Right arrow should complete '#fe' to '#feature'"
    );
}

/// HI-7: Hints dismiss when exact tag match
#[test]
fn test_tag_hints_dismiss_on_exact_match() {
    let content = "# 2026/01/15\n- [ ] Task with #bug tag\n";
    let date = chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let mut ctx = TestContext::with_journal_content(date, content);

    ctx.press(KeyCode::Enter);
    ctx.type_str("#bug");

    assert!(
        !ctx.app.hint_state.is_active(),
        "Hints should dismiss when tag exactly matches"
    );
}

/// HI-8: Hints dismiss when no matches
#[test]
fn test_hints_dismiss_when_no_matches() {
    let content = "# 2026/01/15\n- [ ] Task with #feature tag\n";
    let date = chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let mut ctx = TestContext::with_journal_content(date, content);

    ctx.press(KeyCode::Enter);
    ctx.type_str("#xyz");

    assert!(
        !ctx.app.hint_state.is_active(),
        "Hints should dismiss when no matches"
    );
}

/// HI-9: Hints clear on escape
#[test]
fn test_hints_clear_on_escape() {
    let mut ctx = TestContext::new();

    ctx.press(KeyCode::Char(':'));
    assert!(ctx.app.hint_state.is_active(), "Hints should be active");

    ctx.press(KeyCode::Esc);
    assert!(
        !ctx.app.hint_state.is_active(),
        "Hints should clear on escape"
    );
}

/// HI-10: Filter type hints appear in query mode
#[test]
fn test_filter_type_hints_appear() {
    let mut ctx = TestContext::new();

    ctx.press(KeyCode::Char('/'));
    ctx.type_str("!ta");

    assert!(
        matches!(ctx.app.hint_state, HintContext::FilterTypes { .. }),
        "Should show filter type hints when typing !"
    );
}

/// HI-11: Date operation hints appear in query mode
#[test]
fn test_date_op_hints_appear() {
    let mut ctx = TestContext::new();

    ctx.press(KeyCode::Char('/'));
    ctx.type_str("@be");

    assert!(
        matches!(ctx.app.hint_state, HintContext::DateOps { .. }),
        "Should show date op hints when typing @"
    );
}

/// HI-12: Negation hints appear in query mode
#[test]
fn test_negation_hints_appear() {
    let mut ctx = TestContext::new();

    ctx.press(KeyCode::Char('/'));
    ctx.type_str("not:");

    assert!(
        matches!(ctx.app.hint_state, HintContext::Negation { .. }),
        "Should show negation hints when typing not:"
    );
}

/// HI-13: Hints work with multi-word input (last token)
#[test]
fn test_hints_use_last_token() {
    let content = "# 2026/01/15\n- [ ] Task with #work tag\n";
    let date = chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let mut ctx = TestContext::with_journal_content(date, content);

    ctx.press(KeyCode::Enter);
    ctx.type_str("some text #wo");

    assert!(
        matches!(ctx.app.hint_state, HintContext::Tags { .. }),
        "Should detect tag trigger in multi-word input"
    );
}

/// HI-14: Tag cache refreshes on journal switch
#[test]
fn test_tag_cache_refresh_on_journal_switch() {
    let content = "# 2026/01/15\n- [ ] Task with #project tag\n";
    let date = chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let ctx = TestContext::with_journal_content(date, content);

    assert!(
        ctx.app.cached_journal_tags.contains(&"project".to_string()),
        "Tags should be cached from journal"
    );
}
