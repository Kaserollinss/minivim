use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, read};
use std::io::{self};
use std::path::Path;

use crate::buffer::Buffer;
use crate::cursor::Cursor;
use crate::input::{
    Action, InsertKind, Motion, Operator, ParseResult, Parser, SimpleAction, Target, MotionLocation
};
use crate::pos::Pos;
use crate::terminal::Terminal;
use crate::view::View;

pub struct Editor {
    buffer: Buffer,
    cursor: Cursor,
    view: View,
    parser: Parser,
    should_quit: bool,
    mode: Mode,
}

pub enum Mode {
    Normal,
    Insert,
    Visual,
    Command,
}

impl Editor {
    pub fn default() -> Self {
        Editor {
            buffer: Buffer::empty(),
            cursor: Cursor::default(),
            view: View::new(),
            parser: Parser::new(),
            should_quit: false,
            mode: Mode::Normal,
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        Ok(Editor {
            buffer: Buffer::from_file(path)?,
            ..Editor::default()
        })
    }
    pub fn run(&mut self) {
        Terminal::initialize();
        self.view.render(&self.buffer, &self.cursor);
        let result = self.repl();
        self.view.render_farewell();
        Terminal::terminate();
        result.unwrap();
    }

    fn repl(&mut self) -> Result<(), std::io::Error> {
        loop {
            let event = read()?;
            match event {
                Event::Resize(w, h) => self.view.resize(w, h),
                Event::Key(key) => self.handle_key(key),
                _ => {}
            }

            if self.should_quit {
                break;
            }

            self.view.render(&self.buffer, &self.cursor)?;
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        // temporary exit for now
        if key.code == KeyCode::Char('q') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.should_quit = true;
            return;
        }

        //ignore key up events
        if key.kind == KeyEventKind::Release {
            return;
        }

        if matches!(self.mode, Mode::Insert) {
            self.insert_key(key);
        } else {
            match self.parser.feed(key) {
                ParseResult::Complete(action) => self.apply(action),
                ParseResult::Pending => {}
                ParseResult::Invalid => {} // later: bell, or clear a pending-command display
            }
        }
    }

    fn insert_key(&mut self, key: KeyEvent) {
        let pos = self.cursor.location();

        if matches!(key.code, KeyCode::Esc) {
            self.mode = Mode::Normal;
            return;
        }

        match key {
            KeyEvent {
                code: KeyCode::Backspace,
                ..
            } => {
                //Broken. Time to actually figure out the byte vs char index stuff
                self.buffer.remove_char_at(pos);
            }
            KeyEvent {
                code: KeyCode::Delete,
                ..
            } => {}
            KeyEvent {
                code: KeyCode::Enter,
                ..
            } => self.new_line_below(pos.row),
            KeyEvent {
                code: KeyCode::Left,
                ..
            } => self.cursor.move_left(),
            KeyEvent {
                code: KeyCode::Right,
                ..
            } => self.cursor.move_right(&self.buffer),

            KeyEvent {
                code: KeyCode::Up, ..
            } => self.cursor.move_up(&self.buffer),

            KeyEvent {
                code: KeyCode::Down,
                ..
            } => self.cursor.move_down(&self.buffer),
            KeyEvent {
                code: KeyCode::Esc, ..
            } => {
                self.mode = Mode::Normal;
                Terminal::set_cursor_block();
            }

            _ => {
                let KeyCode::Char(c) = key.code else {
                    return;
                };

                self.buffer.insert_char_at(pos.row, pos.col, c);
                self.cursor.move_right(&self.buffer);
            }
        }
    }

    fn apply(&mut self, action: Action) {
        match action {
            Action::Move(motion, count) => self.handle_movement(motion, count),
            Action::Operate { op, target } => self.handle_operation(op, target),
            Action::EnterInsert(kind) => self.enter_insert(kind),
            Action::EnterVisual => self.mode = Mode::Visual,
            Action::EnterCommandLine => self.mode = Mode::Command,
            Action::Simple(s) => self.handle_simple(s),
        }
    }

    fn handle_movement(&mut self, motion: Motion, count: usize) {
        match motion {
            Motion::Left => self.cursor.move_left(),
            Motion::Right => self.cursor.move_right(&self.buffer),
            Motion::Up => self.cursor.move_up(&self.buffer),
            Motion::Down => self.cursor.move_down(&self.buffer),
            // not implemented yet
            Motion::Word { .. } => self.handle_word_movement(motion, count),
            Motion::Line { .. } => self.handle_line_movement(motion, count),
            Motion::FirstNonBlank => {}
            Motion::File { .. } => self.handle_file_movement(motion, count),
            Motion::Find { .. } => self.handle_find_movement(motion, count),
        }
    }

    fn handle_operation(&mut self, _op: Operator, _target: Target) {}

    fn handle_word_movement(&mut self, motion: Motion, _count: usize) {
        match motion {
            Motion::Word {location: MotionLocation::Start, direction, big} => self.cursor.to_word_start(&self.buffer, direction, big),
            Motion::Word {location: MotionLocation::End, direction, big} => self.cursor.to_word_end(&self.buffer, direction, big),
            _ => {}
        }
    }

    fn handle_line_movement(&mut self, motion: Motion, _count: usize) {
        match motion {
            Motion::Line { location: MotionLocation::Start } => self.cursor.to_line_start(&self.buffer),
            Motion::Line { location: MotionLocation::End } => self.cursor.to_line_end(&self.buffer),
            _ => {}

        }
    }
    fn handle_file_movement(&mut self, motion: Motion, _count: usize) {}
    fn handle_find_movement(&mut self, motion: Motion, _count: usize) {}

    fn enter_insert(&mut self, kind: InsertKind) {
        let pos = self.cursor.location();
        self.mode = Mode::Insert;
        Terminal::set_cursor_vert();

        match kind {
            InsertKind::Before => {}
            InsertKind::After => {}
            InsertKind::LineStart => {}
            InsertKind::LineEnd => {}
            InsertKind::OpenBelow => self.new_line_below(pos.row),
            InsertKind::OpenAbove => self.new_line_above(pos.row),
        }
    }

    fn handle_simple(&mut self, _s: SimpleAction) {}

    fn go_to_new_line_at(&mut self, row: usize) {
        self.buffer.new_line_at(row);
        self.cursor.go_to(Pos::new(row, 0));
    }

    fn new_line_above(&mut self, row: usize) {
        self.go_to_new_line_at(row);
    }

    fn new_line_below(&mut self, row: usize) {
        self.go_to_new_line_at(row + 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    /// Editor over a known buffer, cursor at 0,0. No terminal touched.
    fn editor_with(lines: &[&str]) -> Editor {
        Editor {
            buffer: Buffer::from_lines(lines),
            ..Editor::default()
        }
    }

    fn press(editor: &mut Editor, c: char) {
        editor.handle_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
    }

    #[test]
    fn j_moves_cursor_down() {
        let mut editor = editor_with(&["aaaa", "bb", "cccc"]);
        assert_eq!(editor.cursor.row(), 0);
        press(&mut editor, 'j');
        assert_eq!(editor.cursor.row(), 1);
    }

    #[test]
    fn k_at_top_stays_put() {
        let mut editor = editor_with(&["aaaa", "bb", "cccc"]);
        press(&mut editor, 'k');
        assert_eq!(editor.cursor.row(), 0);
    }

    #[test]
    fn j_at_bottom_stays_put() {
        let mut editor = editor_with(&["aaaa", "bb", "cccc"]);
        press(&mut editor, 'j');
        press(&mut editor, 'j');
        press(&mut editor, 'j'); // already on the last line
        assert_eq!(editor.cursor.row(), 2);
    }

    #[test]
    fn j_onto_shorter_line_clamps_column() {
        let mut editor = editor_with(&["aaaa", "bb", "cccc"]);
        // walk to the end-of-line slot on "aaaa" (col 4)
        for _ in 0..4 {
            press(&mut editor, 'l');
        }
        assert_eq!(editor.cursor.col(), 4);
        // down onto "bb" (len 2) must clamp the column
        press(&mut editor, 'j');
        assert_eq!(editor.cursor.row(), 1);
        assert_eq!(editor.cursor.col(), 2);
    }
}
