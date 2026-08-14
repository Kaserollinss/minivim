use crate::buffer::Buffer;
use crate::cursor::Cursor;
use crate::terminal::Terminal;
use std::io::Error;

pub struct View {
    size: Size,
    /// Top-left buffer cell currently visible — i.e. the scroll position.
    offset: Position,
    needs_redraw: bool,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub col: usize,
    pub row: usize,
}

impl View {
    pub fn new() -> Self {
        let (width, height) = Terminal::size().unwrap_or((80, 24));
        View {
            size: Size {
                width: width as usize,
                height: height as usize,
            },
            offset: Position::default(),
            needs_redraw: true,
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.size = Size {
            width: width as usize,
            height: height as usize,
        };
        self.needs_redraw = true;
    }

    /// Anything that changes what's on screen calls this; `render` is a no-op
    /// otherwise, so idle keys don't cause a repaint.
    pub fn mark_dirty(&mut self) {
        self.needs_redraw = true;
    }

    /// Rows available for buffer text — the last row is the status bar.
    fn text_height(&self) -> usize {
        self.size.height.saturating_sub(1)
    }

    pub fn render(&mut self, buffer: &Buffer, cursor: &Cursor) -> Result<(), Error> {
        if !self.needs_redraw || self.size.height == 0 {
            return Ok(());
        }

        self.scroll_into_view(cursor);

        Terminal::hide_cursor()?;
        for screen_row in 0..self.text_height() {
            Terminal::move_to(0, screen_row as u16)?;
            Terminal::clear_line()?;
            match buffer.line(self.offset.row + screen_row) {
                Some(line) => Terminal::print(self.visible_slice(line))?,
                None => Terminal::print("~")?,
            }
        }
        self.draw_status_bar(buffer, cursor)?;

        let screen = self.to_screen(cursor);
        Terminal::move_to(screen.col as u16, screen.row as u16)?;
        Terminal::show_cursor()?;
        Terminal::flush()?;

        self.needs_redraw = false;
        Ok(())
    }

    /// Draw the goodbye frame on exit.
    pub fn render_farewell(&self) -> Result<(), Error> {
        Terminal::clear_screen()?;
        Terminal::move_to(0, 0)?;
        Terminal::print("Goodbye.\r\n")?;
        Terminal::flush()
    }

    /// Adjust `offset` so the cursor stays on screen. Four cases: off the top,
    /// off the bottom, off the left, off the right.
    fn scroll_into_view(&mut self, cursor: &Cursor) {
        let (row, col) = (cursor.row(), cursor.col());
        let height = self.text_height();

        if row < self.offset.row {
            self.offset.row = row;
        } else if height > 0 && row >= self.offset.row + height {
            self.offset.row = row - height + 1;
        }

        if col < self.offset.col {
            self.offset.col = col;
        } else if self.size.width > 0 && col >= self.offset.col + self.size.width {
            self.offset.col = col - self.size.width + 1;
        }
    }

    /// Buffer coordinates -> screen coordinates.
    fn to_screen(&self, cursor: &Cursor) -> Position {
        Position {
            col: cursor.col().saturating_sub(self.offset.col),
            row: cursor.row().saturating_sub(self.offset.row),
        }
    }

    /// The horizontal window into a line, for when it is scrolled or overlong.
    ///
    /// Sliced on character boundaries rather than bytes so multi-byte text does
    /// not panic. Wide characters still misalign — that needs display widths.
    fn visible_slice<'a>(&self, line: &'a str) -> &'a str {
        let start = line
            .char_indices()
            .nth(self.offset.col)
            .map_or(line.len(), |(i, _)| i);
        let end = line
            .char_indices()
            .nth(self.offset.col + self.size.width)
            .map_or(line.len(), |(i, _)| i);
        &line[start..end]
    }

    fn draw_status_bar(&self, buffer: &Buffer, cursor: &Cursor) -> Result<(), Error> {
        Terminal::move_to(0, self.text_height() as u16)?;
        Terminal::clear_line()?;

        let name = buffer
            .filename()
            .and_then(|p| p.file_name())
            .map_or("[No Name]".to_string(), |n| n.to_string_lossy().into_owned());
        let modified = if buffer.is_modified() { " [+]" } else { "" };
        let left = format!("{name}{modified} — {} lines", buffer.len());
        let right = format!("{}:{}", cursor.row() + 1, cursor.col() + 1);

        let gap = self
            .size
            .width
            .saturating_sub(left.len() + right.len())
            .max(1);
        let status = format!("{left}{}{right}", " ".repeat(gap));
        Terminal::print(self.truncate(&status))
    }

    fn truncate<'a>(&self, s: &'a str) -> &'a str {
        match s.char_indices().nth(self.size.width) {
            Some((i, _)) => &s[..i],
            None => s,
        }
    }
}

impl Default for View {
    fn default() -> Self {
        Self::new()
    }
}
