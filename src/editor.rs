use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, read};
use std::io::{self};
use std::path::Path;

use crate::buffer::Buffer;
use crate::cursor::Cursor;
use crate::input::{
    Action, InsertKind, Motion, Operator, ParseResult, Parser, SimpleAction, Target,
};
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
            self.insert_key(key)
        } else {
            match self.parser.feed(key) {
                ParseResult::Complete(action) => self.apply(action),
                ParseResult::Pending => {}
                ParseResult::Invalid => {} // later: bell, or clear a pending-command display
            }
        }
    }

    fn insert_key(&mut self, key: KeyEvent) {
        if matches!(key.code, KeyCode::Esc) {
            self.mode = Mode::Normal;
            return;
        }

        let KeyCode::Char(c) = key.code else {
            return;
        };

        // This is not perfect and has quirks but for initial implementation im fine with it.
        // some things to consider are inserting new lines or using arrow keys for nav in insert mode.
        let pos = self.cursor.location();
        self.buffer.insert_char_at(pos.row, pos.col, c);
        self.cursor.move_right(&self.buffer);
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

    fn handle_movement(&mut self, motion: Motion, _count: usize) {
        match motion {
            Motion::Left => self.cursor.move_left(),
            Motion::Right => self.cursor.move_right(&self.buffer),
            Motion::Up => self.cursor.move_up(&self.buffer),
            Motion::Down => self.cursor.move_down(&self.buffer),
            // not implemented yet
            Motion::Word { .. } => {}
            Motion::Line { .. } => {}
            Motion::FirstNonBlank => {}
            Motion::File { .. } => {}
            Motion::Find { .. } => {}
        }
    }

    fn handle_operation(&mut self, _op: Operator, _target: Target) {}

    fn enter_insert(&mut self, _kind: InsertKind) {}

    fn handle_simple(&mut self, _s: SimpleAction) {}
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
