use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line as RatatuiLine, Span},
};

use crate::app::{InputMode, InterfaceContext, PromptContext, ViewMode};
use crate::dispatch::Keymap;
use crate::registry::{FooterMode, KeyAction, KeyContext, footer_actions};

use super::shared::format_key_for_display;
use super::theme;

pub struct FooterModel<'a> {
    pub view: &'a ViewMode,
    pub input_mode: &'a InputMode,
    pub keymap: &'a Keymap,
}

impl<'a> FooterModel<'a> {
    #[must_use]
    pub fn new(view: &'a ViewMode, input_mode: &'a InputMode, keymap: &'a Keymap) -> Self {
        Self {
            view,
            input_mode,
            keymap,
        }
    }
}

pub fn render_footer(model: FooterModel<'_>) -> RatatuiLine<'static> {
    match (model.view, model.input_mode) {
        (_, InputMode::Prompt(PromptContext::Command { buffer })) => {
            prompt_line(":", theme::PROMPT_COMMAND, buffer.content())
        }
        (_, InputMode::Prompt(PromptContext::Filter { buffer })) => {
            prompt_line("/", theme::PROMPT_FILTER, buffer.content())
        }
        (_, InputMode::Prompt(PromptContext::RenameTag { old_tag, buffer })) => {
            RatatuiLine::from(vec![
                Span::styled("Rename ", Style::default().fg(theme::PROMPT_RENAME_TAG)),
                Span::styled(
                    format!("#{}", old_tag),
                    Style::default().fg(theme::PROMPT_TAG_HIGHLIGHT),
                ),
                Span::styled(" to: ", Style::default().fg(theme::PROMPT_RENAME_TAG)),
                Span::raw(buffer.content().to_string()),
            ])
        }
        (_, InputMode::Edit(_)) => {
            build_footer_line(" EDIT ", theme::MODE_EDIT, FooterMode::Edit, model.keymap)
        }
        (_, InputMode::Reorder) => build_footer_line(
            " REORDER ",
            theme::MODE_REORDER,
            FooterMode::Reorder,
            model.keymap,
        ),
        (_, InputMode::Confirm(_)) => RatatuiLine::from(vec![
            Span::styled(
                " CONFIRM ",
                Style::default()
                    .fg(theme::FOOTER_INVERSE_FG)
                    .bg(theme::MODE_INTERFACE),
            ),
            Span::styled("  y", Style::default().fg(theme::FOOTER_HINT)),
            Span::styled(" Yes  ", Style::default().dim()),
            Span::styled("n/Esc", Style::default().fg(theme::FOOTER_HINT)),
            Span::styled(" No", Style::default().dim()),
        ]),
        (_, InputMode::Selection(state)) => {
            let count = state.count();
            build_footer_line(
                &format!(" SELECT ({count}) "),
                theme::MODE_SELECTION,
                FooterMode::Selection,
                model.keymap,
            )
        }
        (_, InputMode::Interface(InterfaceContext::Date(_))) => build_footer_line(
            " DATE ",
            theme::MODE_INTERFACE,
            FooterMode::DateInterface,
            model.keymap,
        ),
        (_, InputMode::Interface(InterfaceContext::Project(_))) => build_footer_line(
            " PROJECT ",
            theme::MODE_INTERFACE,
            FooterMode::ProjectInterface,
            model.keymap,
        ),
        (_, InputMode::Interface(InterfaceContext::Tag(_))) => build_footer_line(
            " TAG ",
            theme::MODE_INTERFACE,
            FooterMode::TagInterface,
            model.keymap,
        ),
        (ViewMode::Daily(_), InputMode::Normal) => build_footer_line(
            " DAILY ",
            theme::MODE_DAILY,
            FooterMode::NormalDaily,
            model.keymap,
        ),
        (ViewMode::Filter(_), InputMode::Normal) => build_footer_line(
            " FILTER ",
            theme::MODE_FILTER,
            FooterMode::NormalFilter,
            model.keymap,
        ),
    }
}

fn footer_mode_to_context(mode: FooterMode) -> KeyContext {
    match mode {
        FooterMode::NormalDaily => KeyContext::DailyNormal,
        FooterMode::NormalFilter => KeyContext::FilterNormal,
        FooterMode::Edit => KeyContext::Edit,
        FooterMode::Reorder => KeyContext::Reorder,
        FooterMode::Selection => KeyContext::Selection,
        FooterMode::DateInterface => KeyContext::DateInterface,
        FooterMode::ProjectInterface => KeyContext::ProjectInterface,
        FooterMode::TagInterface => KeyContext::TagInterface,
    }
}

fn build_footer_line(
    mode_name: &str,
    color: Color,
    mode: FooterMode,
    keymap: &Keymap,
) -> RatatuiLine<'static> {
    let mut spans = vec![Span::styled(
        mode_name.to_string(),
        Style::default().fg(theme::FOOTER_INVERSE_FG).bg(color),
    )];

    let context = footer_mode_to_context(mode);

    for action in footer_actions(mode) {
        spans.extend(action_spans(action, keymap, context));
    }

    RatatuiLine::from(spans)
}

fn prompt_line(prefix: &str, color: Color, content: &str) -> RatatuiLine<'static> {
    RatatuiLine::from(vec![
        Span::styled(prefix.to_string(), Style::default().fg(color)),
        Span::raw(content.to_string()),
    ])
}

fn action_spans(action: &KeyAction, keymap: &Keymap, context: KeyContext) -> [Span<'static>; 2] {
    let keys = keymap.keys_for_action_ordered(context, action.id);

    let key_display = if keys.is_empty() {
        // Fall back to default_keys if no keys bound (shouldn't happen normally)
        match action.default_keys {
            [first, second, ..] => {
                format!(
                    "{}/{}",
                    format_key_for_display(first),
                    format_key_for_display(second)
                )
            }
            [first] => format_key_for_display(first),
            [] => String::new(),
        }
    } else if keys.len() == 1 {
        format_key_for_display(&keys[0])
    } else {
        format!(
            "{}/{}",
            format_key_for_display(&keys[0]),
            format_key_for_display(&keys[1])
        )
    };

    [
        Span::styled(
            format!("  {key_display}"),
            Style::default().fg(theme::FOOTER_KEY),
        ),
        Span::styled(format!(" {} ", action.footer_text), Style::default().dim()),
    ]
}

pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}
