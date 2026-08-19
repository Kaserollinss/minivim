use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, read};
use std::io::{self};
use std::path::Path;

use crate::buffer::Buffer;
use crate::cursor::Cursor;
use crate::input::{
    Action, InsertKind, Motion, Operator, ParseResult, Parser, SimpleAction, Target,
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
        dbg_log!("editor started");
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

        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                Terminal::set_cursor_block();
            }
            KeyCode::Backspace => match pos.col {
                0 => {
                    let end_of_target_line = self.buffer.line_len(pos.row - 1);
                    self.buffer.append_lines(pos.row, pos.row - 1);
                    self.cursor.go_to(Pos::new(pos.row - 1, end_of_target_line));
                }
                _ => {
                    self.buffer.remove_char_at(Pos::new(pos.row, pos.col - 1));
                    // move the cursor back one position
                    self.cursor.move_left();
                }
            },
            // when on pos col: 0 you shouldn't delete the entire line.
            KeyCode::Delete => match pos.col {
                col if col == self.buffer.line_len(pos.row) => {
                    let end_of_target_line = self.buffer.line_len(pos.row);
                    self.buffer.append_lines(pos.row + 1, pos.row);
                    self.cursor.go_to(Pos::new(pos.row, end_of_target_line));
                }
                _ => {
                    self.buffer.remove_char_at(Pos::new(pos.row, pos.col));
                }
            },
            KeyCode::Enter => self.new_line_below(pos.row),
            KeyCode::Left => self.cursor.move_left(),
            KeyCode::Right => self.cursor.move_right(&self.buffer),
            KeyCode::Up => self.cursor.move_up(&self.buffer),
            KeyCode::Down => self.cursor.move_down(&self.buffer),
            // Ctrl-chords arrive as plain `Char`s; typing one must not insert its letter.
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.buffer.insert_char_at(pos.row, pos.col, c);
                self.cursor.move_right(&self.buffer);
            }
            _ => {}
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

    fn handle_movement(&mut self, motion: Motion, _count: usize) {
        match motion {
            Motion::Left => self.cursor.move_left(),
            Motion::Right => self.cursor.move_right(&self.buffer),
            Motion::Up => self.cursor.move_up(&self.buffer),
            Motion::Down => self.cursor.move_down(&self.buffer),
            Motion::Word {
                direction,
                location,
                big,
            } => self.cursor.to_word(&self.buffer, direction, location, big),
            Motion::LineStart => self.cursor.to_line_start(),
            Motion::LineEnd => self.cursor.to_line_end(&self.buffer),
            // not implemented yet
            Motion::FirstNonBlank => {}
            Motion::FileEnd => {
                let last_line = self.buffer.line(self.buffer.len() - 2);
                if let Some(last_line) = last_line {
                    self.cursor
                        .go_to(Pos::new(self.buffer.len() - 1, last_line.len() - 1))
                }
            }
            Motion::FileStart => self.cursor.go_to(Pos::new(0, 0)),
            Motion::Find { .. } => {}
        }
    }

    fn handle_operation(&mut self, _op: Operator, _target: Target) {}

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
