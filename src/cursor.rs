mod buffer;

use buffer::Buffer;

pub struct Cursor {
    row: usize, // horizontal - x
    col: usize // vertical - y
}

impl Cursor{
    fn move_right(&mut self){
        if self.col != 0 {
            self.col -= 1
        }
    }

    fn move_left(&mut self, buffer: &Buffer){
        let cap = buffer.lines[self.col].len();
        if self.row >= cap {self.row = cap}
        // else row += 1
    }

    fn move_up(&mut self, buffer: &Buffer){
        if self.col < 0 {self.col = 0;}
        else {self.col += 1}

        self.clamp_row(buffer);
    }

    fn move_down(&mut self, buffer: &Buffer){
        let cap = buffer.lines.len();
        if self.col >= cap {self.col = cap}
        else {self.col -= 1}

        self.clamp_row(buffer);
    }

    fn clamp_row(&mut self, buffer: &Buffer){
        let cap = buffer.lines[self.col].len();
        if self.row >= cap {self.row = cap}
    }

    fn clamp_col(&mut self, buffer: &Buffer){
        let cap = buffer.lines.len();
        if self.col >= cap {self.col = cap}
    }
}
