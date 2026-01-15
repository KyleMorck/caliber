mod helpers;

use chrono::NaiveDate;
use crossterm::event::{KeyCode, KeyModifiers};
use helpers::TestContext;

use caliber::app::InputMode;

#[test]
fn v_key_enters_and_escape_exits_selection() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] Entry A\n- [ ] Entry B\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    ctx.press(KeyCode::Char('v'));
    assert!(matches!(ctx.app.input_mode, InputMode::Selection(_)));

    ctx.press(KeyCode::Esc);
    assert!(matches!(ctx.app.input_mode, InputMode::Normal));
}

#[test]
fn v_key_toggles_entry_selection() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] Entry A\n- [ ] Entry B\n- [ ] Entry C\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    ctx.press(KeyCode::Char('g'));
    ctx.press(KeyCode::Char('v'));

    let state = ctx.app.get_selection_state().unwrap();
    assert!(state.is_selected(0));
    assert_eq!(state.count(), 1);

    ctx.press(KeyCode::Char('j'));
    ctx.press(KeyCode::Char('v'));

    let state = ctx.app.get_selection_state().unwrap();
    assert!(state.is_selected(0));
    assert!(state.is_selected(1));
    assert_eq!(state.count(), 2);
}

#[test]
fn shift_v_selects_range_of_entries() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] Entry A\n- [ ] Entry B\n- [ ] Entry C\n- [ ] Entry D\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    ctx.press(KeyCode::Char('g'));
    ctx.press(KeyCode::Char('v'));

    ctx.press(KeyCode::Char('G'));

    ctx.press_with_modifiers(KeyCode::Char('V'), KeyModifiers::SHIFT);

    let state = ctx.app.get_selection_state().unwrap();
    assert_eq!(state.count(), 4);
    assert!(state.is_selected(0));
    assert!(state.is_selected(1));
    assert!(state.is_selected(2));
    assert!(state.is_selected(3));
}

#[test]
fn shift_v_double_tap_deselects_range() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] Entry A\n- [ ] Entry B\n- [ ] Entry C\n- [ ] Entry D\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    ctx.press(KeyCode::Char('g'));
    ctx.press(KeyCode::Char('v'));
    ctx.press(KeyCode::Char('G'));

    ctx.press_with_modifiers(KeyCode::Char('V'), KeyModifiers::SHIFT);
    assert_eq!(ctx.app.get_selection_state().unwrap().count(), 4);

    ctx.press_with_modifiers(KeyCode::Char('V'), KeyModifiers::SHIFT);
    assert_eq!(ctx.app.get_selection_state().unwrap().count(), 0);
}

#[test]
fn range_select_anchor_updates_on_move() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] A\n- [ ] B\n- [ ] C\n- [ ] D\n- [ ] E\n- [ ] F\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    ctx.press(KeyCode::Char('g'));
    ctx.press(KeyCode::Char('v'));
    ctx.press(KeyCode::Char('j'));
    ctx.press(KeyCode::Char('j'));
    ctx.press(KeyCode::Char('j'));

    ctx.press_with_modifiers(KeyCode::Char('V'), KeyModifiers::SHIFT);
    let state = ctx.app.get_selection_state().unwrap();
    assert_eq!(state.count(), 4);
    assert!(state.is_selected(0));
    assert!(state.is_selected(3));

    ctx.press(KeyCode::Char('j'));
    ctx.press(KeyCode::Char('j'));

    ctx.press_with_modifiers(KeyCode::Char('V'), KeyModifiers::SHIFT);
    let state = ctx.app.get_selection_state().unwrap();
    assert_eq!(state.count(), 6);
    assert!(state.is_selected(4));
    assert!(state.is_selected(5));
}

#[test]
fn deselect_then_move_keeps_original_anchor() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] A\n- [ ] B\n- [ ] C\n- [ ] D\n- [ ] E\n- [ ] F\n- [ ] G\n- [ ] H\n- [ ] I\n- [ ] J\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    ctx.press(KeyCode::Char('G'));
    ctx.press(KeyCode::Char('v'));
    ctx.press(KeyCode::Char('k'));
    ctx.press(KeyCode::Char('k'));
    ctx.press(KeyCode::Char('k'));

    ctx.press_with_modifiers(KeyCode::Char('V'), KeyModifiers::SHIFT);
    assert_eq!(ctx.app.get_selection_state().unwrap().count(), 4);

    ctx.press_with_modifiers(KeyCode::Char('V'), KeyModifiers::SHIFT);
    assert_eq!(ctx.app.get_selection_state().unwrap().count(), 0);

    ctx.press(KeyCode::Char('k'));
    ctx.press(KeyCode::Char('k'));
    ctx.press(KeyCode::Char('k'));

    ctx.press_with_modifiers(KeyCode::Char('V'), KeyModifiers::SHIFT);
    let state = ctx.app.get_selection_state().unwrap();
    assert_eq!(state.count(), 7);
    assert!(state.is_selected(3));
    assert!(state.is_selected(9));
}

#[test]
fn d_key_batch_deletes_selected_entries() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] Keep\n- [ ] Delete A\n- [ ] Delete B\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    ctx.press(KeyCode::Char('g'));
    ctx.press(KeyCode::Char('j'));

    ctx.press(KeyCode::Char('v'));

    ctx.press(KeyCode::Char('j'));
    ctx.press(KeyCode::Char('v'));

    ctx.press(KeyCode::Char('d'));

    assert!(matches!(ctx.app.input_mode, InputMode::Normal));

    let journal = ctx.read_journal();
    assert!(journal.contains("Keep"));
    assert!(!journal.contains("Delete A"));
    assert!(!journal.contains("Delete B"));
}

#[test]
fn c_key_batch_toggles_selected_tasks() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] Task A\n- [ ] Task B\n- Note\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    ctx.press(KeyCode::Char('g'));
    ctx.press(KeyCode::Char('v'));

    ctx.press(KeyCode::Char('j'));
    ctx.press(KeyCode::Char('v'));

    ctx.press(KeyCode::Char(' '));

    let journal = ctx.read_journal();
    assert!(journal.contains("[x] Task A"));
    assert!(journal.contains("[x] Task B"));
}

#[test]
fn y_key_batch_yanks_selected_entries() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] Entry A\n- [ ] Entry B\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    ctx.press(KeyCode::Char('g'));
    ctx.press(KeyCode::Char('v'));

    ctx.press(KeyCode::Char('j'));
    ctx.press(KeyCode::Char('v'));

    ctx.press(KeyCode::Char('y'));

    assert!(matches!(ctx.app.input_mode, InputMode::Selection(_)));

    assert!(ctx.screen_contains("Entry A"));
    assert!(ctx.screen_contains("Entry B"));
}

#[test]
fn selection_respects_hidden_completed_entries() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] Incomplete A\n- [x] Completed\n- [ ] Incomplete B\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    ctx.press(KeyCode::Char('z'));

    ctx.press(KeyCode::Char('g'));
    ctx.press(KeyCode::Char('v'));

    ctx.press(KeyCode::Char('j'));
    ctx.press(KeyCode::Char('v'));

    ctx.press(KeyCode::Char('d'));

    let journal = ctx.read_journal();
    assert!(!journal.contains("Incomplete A"));
    assert!(journal.contains("Completed"));
    assert!(!journal.contains("Incomplete B"));
}

#[test]
fn x_key_removes_last_tag_from_selected() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] Entry A #tag1 #tag2\n- [ ] Entry B #tag3\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    ctx.press(KeyCode::Char('g'));
    ctx.press(KeyCode::Char('v'));

    ctx.press(KeyCode::Char('j'));
    ctx.press(KeyCode::Char('v'));

    ctx.press(KeyCode::Char('x'));

    let journal = ctx.read_journal();
    assert!(journal.contains("Entry A #tag1") && !journal.contains("#tag2"));
    assert!(!journal.contains("#tag3"));
}

#[test]
fn shift_x_removes_all_tags_from_selected() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] Entry A #tag1 #tag2\n- [ ] Entry B #tag3\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    ctx.press(KeyCode::Char('g'));
    ctx.press(KeyCode::Char('v'));

    ctx.press(KeyCode::Char('j'));
    ctx.press(KeyCode::Char('v'));

    ctx.press(KeyCode::Char('X'));

    let journal = ctx.read_journal();
    assert!(!journal.contains("#tag1") && !journal.contains("#tag2"));
    assert!(!journal.contains("#tag3"));
}

#[test]
fn batch_delete_undos_as_single_operation() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] Keep\n- [ ] Delete A\n- [ ] Delete B\n- [ ] Delete C\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    ctx.press(KeyCode::Char('g'));
    ctx.press(KeyCode::Char('j'));

    ctx.press(KeyCode::Char('v'));

    ctx.press(KeyCode::Char('G'));
    ctx.press_with_modifiers(KeyCode::Char('V'), KeyModifiers::SHIFT);

    let state = ctx.app.get_selection_state().unwrap();
    assert_eq!(state.count(), 3);

    ctx.press(KeyCode::Char('d'));

    let journal = ctx.read_journal();
    assert!(journal.contains("Keep"));
    assert!(!journal.contains("Delete A"));
    assert!(!journal.contains("Delete B"));
    assert!(!journal.contains("Delete C"));

    ctx.press(KeyCode::Char('u'));

    let journal = ctx.read_journal();
    assert!(journal.contains("Keep"));
    assert!(journal.contains("Delete A"));
    assert!(journal.contains("Delete B"));
    assert!(journal.contains("Delete C"));
}

#[test]
fn batch_delete_supports_undo_redo() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] Keep\n- [ ] Delete Me\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    ctx.press(KeyCode::Char('G'));
    ctx.press(KeyCode::Char('d'));

    let journal = ctx.read_journal();
    assert!(!journal.contains("Delete Me"));

    ctx.press(KeyCode::Char('u'));
    let journal = ctx.read_journal();
    assert!(journal.contains("Delete Me"));

    ctx.press(KeyCode::Char('U'));
    let journal = ctx.read_journal();
    assert!(!journal.contains("Delete Me"));

    ctx.press(KeyCode::Char('u'));
    let journal = ctx.read_journal();
    assert!(journal.contains("Delete Me"));
}

#[test]
fn selection_mode_allows_navigation_keys() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content = "# 2026/01/15\n- [ ] A\n- [ ] B\n- [ ] C\n- [ ] D\n";
    let mut ctx = TestContext::with_journal_content(date, content);

    ctx.press(KeyCode::Char('g'));
    ctx.press(KeyCode::Char('v'));

    ctx.press(KeyCode::Char('j'));
    assert_eq!(ctx.selected_index(), 1);

    ctx.press(KeyCode::Char('k'));
    assert_eq!(ctx.selected_index(), 0);

    ctx.press(KeyCode::Char('G'));
    assert_eq!(ctx.selected_index(), 3);

    ctx.press(KeyCode::Char('g'));
    assert_eq!(ctx.selected_index(), 0);

    assert!(matches!(ctx.app.input_mode, InputMode::Selection(_)));
}

#[test]
fn selection_allows_later_entries() {
    // Later entries (@date) are now selectable, only Recurring entries are blocked
    let view_date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let content =
        "# 2026/01/10\n- [ ] Later task @01/15\n# 2026/01/15\n- [ ] Local A\n- [ ] Local B\n";
    let mut ctx = TestContext::with_journal_content(view_date, content);

    assert!(ctx.screen_contains("Later task"));
    assert!(ctx.screen_contains("Local A"));

    // Later entry is at index 0, should be selectable now
    ctx.press(KeyCode::Char('g'));
    ctx.press(KeyCode::Char('v'));
    assert!(matches!(ctx.app.input_mode, InputMode::Selection(_)));

    // Can navigate freely including Later entry
    ctx.press(KeyCode::Char('j'));
    assert_eq!(ctx.selected_index(), 1);

    ctx.press(KeyCode::Char('k'));
    assert_eq!(ctx.selected_index(), 0); // Can move back to Later entry
}

#[test]
fn selection_skips_recurring_entries() {
    // Recurring entries (@every-*) cannot be selected
    let monday = NaiveDate::from_ymd_opt(2026, 1, 12).unwrap(); // A Monday
    let content = "# 2026/01/05\n- [ ] Weekly standup @every-monday\n# 2026/01/12\n- [ ] Local A\n- [ ] Local B\n";
    let mut ctx = TestContext::with_journal_content(monday, content);

    assert!(ctx.screen_contains("Weekly standup"));
    assert!(ctx.screen_contains("Local A"));

    // Recurring entry is at index 0, should NOT be selectable
    ctx.press(KeyCode::Char('g'));
    ctx.press(KeyCode::Char('v'));
    assert!(matches!(ctx.app.input_mode, InputMode::Normal));

    // Move to first local entry, then selection works
    ctx.press(KeyCode::Char('j'));
    ctx.press(KeyCode::Char('v'));
    assert!(matches!(ctx.app.input_mode, InputMode::Selection(_)));

    // Jump to first should go to first non-recurring entry (index 1)
    ctx.press(KeyCode::Char('g'));
    assert_eq!(ctx.selected_index(), 1);

    // Can't move up into recurring section
    ctx.press(KeyCode::Char('k'));
    assert_eq!(ctx.selected_index(), 1);
}
