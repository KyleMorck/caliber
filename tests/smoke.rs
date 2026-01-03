mod helpers;

use crossterm::event::KeyCode;
use helpers::TestContext;

/// Smoke test covering the critical path: create, navigate, toggle, filter, delete, undo
#[test]
fn smoke_test_core_workflow() {
    let mut ctx = TestContext::new();

    // 1. Create entry
    ctx.press(KeyCode::Enter);
    ctx.type_str("Smoke test entry");
    ctx.press(KeyCode::Enter);

    // Verify entry appears
    assert!(
        ctx.screen_contains("Smoke test entry"),
        "Entry should appear after creation"
    );

    // 2. Navigate (j/k)
    ctx.press(KeyCode::Char('j'));
    ctx.press(KeyCode::Char('k'));

    // 3. Toggle complete
    ctx.press(KeyCode::Char('c'));
    assert!(
        ctx.screen_contains("[x]"),
        "Entry should show completed marker"
    );

    // 4. Filter for the entry
    ctx.press(KeyCode::Char('/'));
    ctx.type_str("Smoke");
    ctx.press(KeyCode::Enter);
    assert!(
        ctx.screen_contains("Smoke test entry"),
        "Entry should appear in filter results"
    );

    // 5. Exit filter
    ctx.press(KeyCode::Tab);

    // 6. Delete entry
    ctx.press(KeyCode::Char('x'));
    assert!(
        !ctx.screen_contains("Smoke test entry"),
        "Entry should be deleted"
    );

    // 7. Undo
    ctx.press(KeyCode::Char('u'));
    assert!(
        ctx.screen_contains("Smoke test entry"),
        "Entry should be restored after undo"
    );

    // 8. Verify persistence
    let journal = ctx.read_journal();
    assert!(
        journal.contains("Smoke test entry"),
        "Entry should be persisted to journal file"
    );
}
