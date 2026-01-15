use std::io;

use chrono::Local;

use crate::cursor::CursorBuffer;
use crate::storage::{ParseContext, parse_date};

use super::{App, DatePickerState, InputMode};

impl App {
    pub fn open_date_picker(&mut self) {
        self.input_mode = InputMode::DatePicker(DatePickerState {
            buffer: CursorBuffer::empty(),
        });
    }

    pub fn submit_date_picker(&mut self) -> io::Result<()> {
        let InputMode::DatePicker(state) = &self.input_mode else {
            return Ok(());
        };

        let input = state.buffer.content().trim().to_string();
        if input.is_empty() {
            self.close_date_picker();
            return Ok(());
        }

        let today = Local::now().date_naive();
        match parse_date(&input, ParseContext::Interface, today) {
            Some(date) => {
                self.input_mode = InputMode::Normal;
                self.goto_day(date)?;
            }
            None => {
                self.set_error(format!("Invalid date: {}", input));
            }
        }
        Ok(())
    }

    pub fn close_date_picker(&mut self) {
        self.input_mode = InputMode::Normal;
    }
}
