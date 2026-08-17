use crossterm::cursor::{Hide, MoveTo, SetCursorStyle, Show};
use crossterm::queue;
use crossterm::style::Print;
use crossterm::terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode, size};
use std::io::{Error, Write, stdout};

pub struct Terminal;

impl Terminal {
    pub fn initialize() -> Result<(), Error> {
        enable_raw_mode()?;
        Self::clear_screen()?;
        Self::move_to(0, 0)?;
        Self::set_cursor_block();
        Self::flush()
    }

    pub fn terminate() -> Result<(), Error> {
        Self::show_cursor()?;
        Self::flush()?;
        disable_raw_mode()
    }

    pub fn clear_screen() -> Result<(), Error> {
        queue!(stdout(), Clear(ClearType::All))
    }

    pub fn clear_line() -> Result<(), Error> {
        queue!(stdout(), Clear(ClearType::CurrentLine))
    }

    pub fn move_to(col: u16, row: u16) -> Result<(), Error> {
        queue!(stdout(), MoveTo(col, row))
    }

    pub fn hide_cursor() -> Result<(), Error> {
        queue!(stdout(), Hide)
    }

    pub fn show_cursor() -> Result<(), Error> {
        queue!(stdout(), Show)
    }

    pub fn print(text: &str) -> Result<(), Error> {
        queue!(stdout(), Print(text))
    }

    pub fn set_cursor_block() -> Result<(), Error> {
        queue!(stdout(), SetCursorStyle::SteadyBlock)
    }

    pub fn set_cursor_vert() -> Result<(), Error> {
        queue!(stdout(), SetCursorStyle::BlinkingBar)
    }

    /// (columns, rows)
    pub fn size() -> Result<(u16, u16), Error> {
        size()
    }

    pub fn flush() -> Result<(), Error> {
        stdout().flush()
    }
}
