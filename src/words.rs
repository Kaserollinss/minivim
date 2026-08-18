use crate::buffer::Buffer;
use crate::cursor::Cursor;
use crate::pos::Pos;
use crate::walker::{Direction, Walker};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CharClass {
    WhiteSpace,
    Word,
    Punctuation,
}

fn classify(c: char, big: bool) -> CharClass {
    match c {
        c if c.is_whitespace() => CharClass::WhiteSpace,
        c if c.is_alphanumeric() || big => CharClass::Word,
        _ => CharClass::Punctuation,
    }
}

fn is_word_start(pos: Pos, buffer: &Buffer, big: bool) -> bool {
    // non whitespace
    // if big, char to left is whitespace
    // else char to left is different class
    let char = buffer.char_at(pos);
    let class = if let Some(char) = char {
        Some(classify(char, big))
    } else {
        None
    };

    if matches!(class, Some(CharClass::WhiteSpace)) {
        return false;
    }

    let prev_pos = buffer.prev_pos(pos);
    match prev_pos {
        Some(pos) => {
            let next_char = buffer.char_at(pos);
            Some(classify(next_char.unwrap(), big)) != class
        }
        None => return false,
    }
}

fn is_word_end(pos: Pos, buffer: &Buffer, big: bool) -> bool {
    let char = buffer.char_at(pos);
    let class = if let Some(char) = char {
        Some(classify(char, big))
    } else {
        None
    };

    if matches!(class, Some(CharClass::WhiteSpace)) {
        return false;
    }

    let next_pos = buffer.next_pos(pos);
    match next_pos {
        Some(pos) => {
            let next_char = buffer.char_at(pos);
            Some(classify(next_char.unwrap(), big)) != class
        }
        None => return false,
    }
}

pub fn to_word_end(cursor: &mut Cursor, buffer: &Buffer, direction: Direction, is_big: bool) {
    let word_end_pos = Walker::default(buffer, cursor.location(), direction).find(|&pos| is_word_end(pos, buffer, is_big));
    if let Some(pos) = word_end_pos {cursor.go_to(pos)};
}

pub fn to_word_start(cursor: &mut Cursor, buffer: &Buffer, direction: Direction, is_big: bool) {
    let word_start_pos = Walker::default(buffer, cursor.location(), direction).find(|&pos| is_word_start(pos, buffer, is_big));
    if let Some(pos) = word_start_pos {cursor.go_to(pos)};
}
