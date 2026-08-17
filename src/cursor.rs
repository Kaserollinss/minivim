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
    class: Option<CharClass>,
    desired_col: usize, // The col we want on vertical moves; honored when the line allows it
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CharClass {
    WhiteSpace,
    Word,
    Punctuation,
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
        self.desired_col = self.pos.col;
    }

    pub fn move_right(&mut self, buffer: &Buffer) {
        if self.pos.col < buffer.line_len(self.pos.row) {
            self.pos.col += 1;
        }
        self.desired_col = self.pos.col;
    }

    pub fn move_up(&mut self, buffer: &Buffer) {
        self.pos.row = self.pos.row.saturating_sub(1);
        self.restore_col(buffer);
    }

    pub fn move_down(&mut self, buffer: &Buffer) {
        if self.pos.row + 1 < buffer.len() {
            self.pos.row += 1;
        }
        self.restore_col(buffer);
    }

    pub fn go_to_new_line_at(&mut self, row: usize) {
        self.pos = Pos { row, col: 0 };
    }

    /// Honor desired_col on the current line, clamping to what the line allows.
    fn restore_col(&mut self, buffer: &Buffer) {
        self.pos.col = self.desired_col.min(buffer.line_len(self.pos.row));
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
    //pub fn forward_word(&mut self, _buffer: &Buffer) {}

    fn classify(&self, c: char, big: bool) -> CharClass {
        match c {
            c if c.is_whitespace() => CharClass::WhiteSpace,
            c if c.is_alphanumeric() || big => CharClass::Word,
            _ => CharClass::Punctuation,
        }
    }

    fn is_word_start(&self, buffer: &Buffer, big: bool) -> bool {
        // non whitespace
        // if big, char to left is whitespace
        // else char to left is different class
        if matches!(self.class, Some(CharClass::WhiteSpace)) {
            return false;
        }

        let prev_pos = buffer.prev_pos(self.pos);
        match prev_pos {
            Some(pos) => {
                let next_char = buffer.char_at(pos);
                Some(self.classify(next_char.unwrap(), big)) != self.class
            }
            None => return false,
        }
    }

    fn is_word_end(&self, buffer: &Buffer, big: bool) -> bool {
        if matches!(self.class, Some(CharClass::WhiteSpace)) {
            return false;
        }

        let next_pos = buffer.next_pos(self.pos);
        match next_pos {
            Some(pos) => {
                let next_char = buffer.char_at(pos);
                Some(self.classify(next_char.unwrap(), big)) != self.class
            }
            None => return false,
        }
    }
}
