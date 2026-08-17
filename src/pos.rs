/// A position in *buffer* coordinates, in char units (never screen coordinates).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    pub row: usize, // vertical   - y - index into buffer.lines
    pub col: usize, // horizontal - x - char index within that line
}

impl Pos {
    pub fn new(row: usize, col: usize) -> Self {
        Pos { row, col }
    }
}
