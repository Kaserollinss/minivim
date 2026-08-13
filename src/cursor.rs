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

    fn move_left(&mut self, &Buffer buffer){
        // cap = buffer[col].length
        // if row >= cap {row = cap}
        // else row += 1
    }

    fn move_up(&mut self, &Buffer buffer){
        // cap = buffer.length
        // if col >= cap {col = cap}
        // else row += 1
    }

    fn move_down(&mut self, &Buffer buffer){
        // cap = buffer.length
        // if col >= cap {col = cap}
        // else row -= 1
    }

    //We need a clamp row functionality when traversing horizontally
}
