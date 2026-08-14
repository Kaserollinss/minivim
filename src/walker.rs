use crate::{buffer::Buffer, pos::Pos};

pub struct Walker<'a> {
    buffer: &'a Buffer,
    pos: Pos,
    direction: Direction,
}

pub enum Direction {
    Forward,
    Backward,
}

impl<'a> Walker<'a> {
    pub fn default(buffer: &'a Buffer, pos: Pos, direction: Direction) -> Self {
        Walker {
            buffer,
            pos,
            direction,
        }
    }

    pub fn walk_from(&self) {}
}
