use crate::{buffer::Buffer, pos::Pos};

pub struct Walker<'a> {
    buffer: &'a Buffer,
    pos: Option<Pos>,
    direction: Direction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Forward,
    Backward,
}

impl<'a> Walker<'a> {
    pub fn default(buffer: &'a Buffer, pos: Pos, direction: Direction) -> Self {
        Walker {
            buffer,
            pos: Some(pos),
            direction,
        }
    }
}

impl Iterator for Walker<'_> {
    type Item = Pos;

    fn next(&mut self) -> Option<Self::Item> {
        let here = self.pos?; // stop once we've stepped off the buffer
        self.pos = match self.direction {
            Direction::Forward => self.buffer.next_pos(here),
            Direction::Backward => self.buffer.prev_pos(here),
        };
        Some(here)
    }
}
