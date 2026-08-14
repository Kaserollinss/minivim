use crate::buffer::Buffer;
use crate::pos::Pos;

/// Cursor position in *buffer* coordinates, never screen coordinates.
///
/// `View` converts to screen coordinates at draw time using its scroll offset;
/// nothing converts back. If the cursor stored screen coordinates, scrolling
/// would silently change which character you are editing.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pos: Pos,
}

impl Cursor {
    pub fn location(&self) -> Pos {
        self.pos
    }
    pub fn row(&self) -> usize {
        self.pos.row
    }
    pub fn col(&self) -> usize {
        self.pos.col
    }

    pub fn move_left(&mut self) {
        self.pos.col = self.pos.col.saturating_sub(1);
    }

    pub fn move_right(&mut self, buffer: &Buffer) {
        if self.pos.col < buffer.line_len(self.pos.row) {
            self.pos.col += 1;
        }
    }

    pub fn move_up(&mut self, buffer: &Buffer) {
        self.pos.row = self.pos.row.saturating_sub(1);
        self.clamp_col(buffer);
    }

    pub fn move_down(&mut self, buffer: &Buffer) {
        if self.pos.row + 1 < buffer.len() {
            self.pos.row += 1;
        }
        self.clamp_col(buffer);
    }

    /// Keep `row` within the buffer.
    fn clamp_row(&mut self, buffer: &Buffer) {
        let last = buffer.len().saturating_sub(1);
        if self.pos.row > last {
            self.pos.row = last;
        }
    }

    /// Keep `col` within the current line. Called after vertical movement,
    /// since the new line may be shorter than the old one.
    fn clamp_col(&mut self, buffer: &Buffer) {
        let cap = buffer.line_len(self.pos.row);
        if self.pos.col > cap {
            self.pos.col = cap;
        }
    }

    /// Re-clamp both axes — for after the buffer changes under the cursor.
    pub fn clamp(&mut self, buffer: &Buffer) {
        self.clamp_row(buffer);
        self.clamp_col(buffer);
    }

    // Ignores punctuation 'W'
    pub fn forward_word(&mut self, buffer: &Buffer){
    }
}
