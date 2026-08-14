use crate::buffer::Buffer;

/// Cursor position in *buffer* coordinates, never screen coordinates.
///
/// `View` converts to screen coordinates at draw time using its scroll offset;
/// nothing converts back. If the cursor stored screen coordinates, scrolling
/// would silently change which character you are editing.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    row: usize, // vertical   - y - index into buffer.lines
    col: usize, // horizontal - x - char index within that line
}

impl Cursor {
    pub fn location(&self) -> (usize, usize) {
        (self.row, self.col)
    }
    pub fn row(&self) -> usize {
        self.row
    }
    pub fn col(&self) -> usize {
        self.col
    }

    pub fn move_left(&mut self) {
        self.col = self.col.saturating_sub(1);
    }

    pub fn move_right(&mut self, buffer: &Buffer) {
        if self.col < buffer.line_len(self.row) {
            self.col += 1;
        }
    }

    pub fn move_up(&mut self, buffer: &Buffer) {
        self.row = self.row.saturating_sub(1);
        self.clamp_col(buffer);
    }

    pub fn move_down(&mut self, buffer: &Buffer) {
        if self.row + 1 < buffer.len() {
            self.row += 1;
        }
        self.clamp_col(buffer);
    }

    /// Keep `row` within the buffer.
    fn clamp_row(&mut self, buffer: &Buffer) {
        let last = buffer.len().saturating_sub(1);
        if self.row > last {
            self.row = last;
        }
    }

    /// Keep `col` within the current line. Called after vertical movement,
    /// since the new line may be shorter than the old one.
    fn clamp_col(&mut self, buffer: &Buffer) {
        let cap = buffer.line_len(self.row);
        if self.col > cap {
            self.col = cap;
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
