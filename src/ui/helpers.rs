use crate::app::App;

pub fn edit_text(app: &App, is_editing: bool, fallback: &str) -> String {
    if is_editing {
        app.edit_buffer
            .as_ref()
            .map(|buffer| buffer.content().to_string())
            .unwrap_or_else(|| fallback.to_string())
    } else {
        fallback.to_string()
    }
}
