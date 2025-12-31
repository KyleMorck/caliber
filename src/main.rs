mod app;
mod config;
mod cursor;
mod handlers;
mod storage;
mod ui;

use std::io;
use std::path::PathBuf;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
};
use unicode_width::UnicodeWidthStr;

use app::{App, Mode};
use config::Config;

fn ensure_selected_visible(
    scroll_offset: &mut usize,
    selected: usize,
    entry_count: usize,
    visible_height: usize,
) {
    if entry_count == 0 {
        *scroll_offset = 0;
        return;
    }
    if selected < *scroll_offset {
        *scroll_offset = selected;
    }
    if selected >= *scroll_offset + visible_height {
        *scroll_offset = selected - visible_height + 1;
    }
}

fn cursor_position_in_wrap(
    text: &str,
    cursor_display_pos: usize,
    max_width: usize,
) -> (usize, usize) {
    use unicode_width::UnicodeWidthStr;

    if max_width == 0 {
        return (0, cursor_display_pos);
    }

    let mut row = 0;
    let mut col = 0;
    let mut current_width = 0;
    let mut chars_processed = 0;

    for ch in text.chars() {
        let ch_width = ch.to_string().width();

        if chars_processed == cursor_display_pos {
            return (row, col);
        }

        if current_width + ch_width > max_width && current_width > 0 {
            row += 1;
            current_width = 0;
        }

        col = current_width;
        current_width += ch_width;
        chars_processed += ch_width;
    }

    // Cursor at end of text
    if current_width >= max_width {
        (row + 1, 0)
    } else {
        (row, current_width)
    }
}

fn main() -> Result<(), io::Error> {
    let args: Vec<String> = std::env::args().collect();

    if args.get(1).map(String::as_str) == Some("init") {
        return match Config::init() {
            Ok(true) => {
                println!(
                    "Created config file at: {}",
                    config::get_config_path().display()
                );
                Ok(())
            }
            Ok(false) => {
                println!(
                    "Config file already exists at: {}",
                    config::get_config_path().display()
                );
                Ok(())
            }
            Err(e) => {
                eprintln!("Failed to create config file: {e}");
                Err(e)
            }
        };
    }

    let cli_file = args.get(1).map(PathBuf::from);
    let config = Config::load().unwrap_or_default();

    let journal_path = if let Some(path) = cli_file {
        if path.is_absolute() {
            path
        } else {
            std::env::current_dir()?.join(path)
        }
    } else {
        config.get_journal_path()
    };

    storage::set_journal_path(journal_path);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {err}");
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let mut app = App::new()?;

    loop {
        let is_tasks_mode = app.mode == Mode::Tasks;

        terminal.draw(|f| {
            let size = f.area();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(1)])
                .split(size);

            let main_block = if is_tasks_mode {
                let incomplete_count = app.task_items.iter().filter(|t| !t.completed).count();
                let tasks_title = format!(" Todos [{}] ", incomplete_count);
                Block::default()
                    .title_top(ratatui::text::Line::from(tasks_title).alignment(Alignment::Right))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::White))
            } else {
                let date_title = app.current_date.format(" %m/%d/%y ").to_string();
                Block::default()
                    .title_top(ratatui::text::Line::from(date_title).alignment(Alignment::Right))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::White))
            };

            let inner = main_block.inner(chunks[0]);

            let content_area = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Min(1),
                    Constraint::Length(1),
                ])
                .split(inner)[1];

            f.render_widget(main_block, chunks[0]);

            let visible_height = content_area.height as usize;

            let content_width = content_area.width as usize;

            if is_tasks_mode {
                let visual_line = app.task_visual_line();
                let total_lines = app.task_total_lines();
                ensure_selected_visible(
                    &mut app.task_scroll_offset,
                    visual_line,
                    total_lines,
                    visible_height,
                );
            } else {
                ensure_selected_visible(
                    &mut app.scroll_offset,
                    app.selected + 1,
                    app.entry_indices.len() + 1,
                    visible_height,
                );
                // Keep header visible when first entry is selected
                if app.selected == 0 {
                    app.scroll_offset = 0;
                }
            }

            let lines = if is_tasks_mode {
                ui::render_tasks_view(&app)
            } else {
                ui::render_daily_view(&app, content_width)
            };

            if !is_tasks_mode
                && app.mode == Mode::Edit
                && let Some(ref buffer) = app.edit_buffer
                && let Some(entry) = app.get_selected_entry()
            {
                let prefix = entry.prefix();
                let prefix_width = prefix.width();
                let text_width = content_width.saturating_sub(prefix_width);

                // Calculate which wrapped line the cursor is on
                let (cursor_row, cursor_col) = cursor_position_in_wrap(
                    buffer.content(),
                    buffer.cursor_display_pos(),
                    text_width,
                );

                // Entry starts at line (selected + 1) accounting for header
                let entry_start_line = app.selected + 1;
                let cursor_line = entry_start_line + cursor_row;

                // Ensure cursor line is visible (may need to scroll for wrapped lines)
                if cursor_line >= app.scroll_offset + visible_height {
                    app.scroll_offset = cursor_line - visible_height + 1;
                }

                if cursor_line >= app.scroll_offset {
                    let screen_row = cursor_line - app.scroll_offset;
                    let col_offset = prefix_width;

                    #[allow(clippy::cast_possible_truncation)]
                    let cursor_x = content_area.x + (col_offset + cursor_col) as u16;
                    #[allow(clippy::cast_possible_truncation)]
                    let cursor_y = content_area.y + screen_row as u16;

                    if cursor_x < content_area.x + content_area.width
                        && cursor_y < content_area.y + content_area.height
                    {
                        f.set_cursor_position((cursor_x, cursor_y));
                    }
                }
            }

            #[allow(clippy::cast_possible_truncation)]
            let scroll_offset = if is_tasks_mode {
                app.task_scroll_offset
            } else {
                app.scroll_offset
            };
            let content = Paragraph::new(lines).scroll((scroll_offset as u16, 0));
            f.render_widget(content, content_area);

            let footer = Paragraph::new(ui::render_footer(&app));
            f.render_widget(footer, chunks[1]);

            // Render help popup overlay
            if app.show_help {
                let popup_area = ui::centered_rect(50, 70, size);
                app.help_visible_height = popup_area.height.saturating_sub(3) as usize;

                f.render_widget(Clear, popup_area);

                let total = ui::get_help_total_lines();
                let max_scroll = total.saturating_sub(app.help_visible_height);
                let can_scroll_up = app.help_scroll > 0;
                let can_scroll_down = app.help_scroll < max_scroll;

                let arrows = match (can_scroll_up, can_scroll_down) {
                    (true, true) => "▲▼",
                    (true, false) => "▲",
                    (false, true) => "▼",
                    (false, false) => "",
                };

                let help_block = Block::default()
                    .title(" Keybindings ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan));

                let inner_area = help_block.inner(popup_area);
                f.render_widget(help_block, popup_area);

                let help_content =
                    ui::render_help_content(app.help_scroll, app.help_visible_height);
                let help_paragraph = Paragraph::new(help_content);
                f.render_widget(help_paragraph, inner_area);

                // Footer spanning bottom of inner area, right-aligned
                let footer_area = ratatui::layout::Rect {
                    x: inner_area.x,
                    y: inner_area.y + inner_area.height.saturating_sub(1),
                    width: inner_area.width,
                    height: 1,
                };
                let footer_line = if arrows.is_empty() {
                    ratatui::text::Line::from(vec![
                        ratatui::text::Span::styled("?", Style::default().fg(Color::White)),
                        ratatui::text::Span::styled(
                            " close ",
                            Style::default().fg(Color::DarkGray),
                        ),
                    ])
                } else {
                    ratatui::text::Line::from(vec![
                        ratatui::text::Span::styled(arrows, Style::default().fg(Color::White)),
                        ratatui::text::Span::styled(
                            " scroll  ",
                            Style::default().fg(Color::DarkGray),
                        ),
                        ratatui::text::Span::styled("?", Style::default().fg(Color::White)),
                        ratatui::text::Span::styled(
                            " close ",
                            Style::default().fg(Color::DarkGray),
                        ),
                    ])
                };
                let footer =
                    Paragraph::new(footer_line).alignment(ratatui::layout::Alignment::Right);
                f.render_widget(footer, footer_area);
            }
        })?;

        if event::poll(std::time::Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            app.status_message = None;

            if app.show_help {
                handlers::handle_help_key(&mut app, key.code);
            } else {
                match app.mode {
                    Mode::Command => handlers::handle_command_key(&mut app, key.code)?,
                    Mode::Daily => handlers::handle_normal_key(&mut app, key.code)?,
                    Mode::Edit => handlers::handle_editing_key(&mut app, key.code),
                    Mode::Tasks => handlers::handle_tasks_key(&mut app, key.code)?,
                    Mode::Order => handlers::handle_order_key(&mut app, key.code),
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
