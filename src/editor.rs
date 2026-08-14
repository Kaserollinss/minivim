use crossterm::event::{read, Event, Event::Key, KeyCode, KeyCode::Char, KeyEvent, KeyModifiers};
use std::io::{self, stdout};
use std::path::{Path};

use crate::buffer::Buffer;
use crate::cursor::Cursor;
use crate::input::{Parser, ParseResult, Action};
use crate::view::View;
use crate::terminal::Terminal;

pub struct Editor {
    buffer: Buffer,
    cursor: Cursor,
    view: View,
    parser: Parser,    
    should_quit: bool,
    mode: Mode,
}

enum Mode {
    Normal,
    Insert,
    Visual,
    Command
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
        let result = self.repl();
        self.view.render_farewell();
        Terminal::terminate();
        result.unwrap();
    }

    fn repl(&mut self) -> Result<(), std::io::Error> {
        loop {
            let event = read()?;
            match event {
                Event::Resize(w,h ) => self.view.resize(w, h),
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

    fn handle_key(&mut self, key: KeyEvent){
        // temporary exit for now
        if key.code == KeyCode::Char('q') && key.modifiers.contains(KeyModifiers::CONTROL)  {
            self.should_quit = true;
            return;
        }
        if matches!(self.mode, Mode::Insert) {
            self.insert_key(key)
        } else {
            match self.parser.feed(key) {
                ParseResult::Complete(action) => self.apply(action),
                ParseResult::Pending => {}
                ParseResult::Invalid => {}   // later: bell, or clear a pending-command display
            }
        }
    }
 
    fn insert_key(&mut self, key: KeyEvent) {
        if matches!(key.code, KeyCode::Esc){
            self.mode = Mode::Normal;
            return;
        }

        let KeyCode::Char(c) = key.code else {
            return;
        };

        // This is not perfect and has quirks but for initial implementation im fine with it. 
        // some things to consider are inserting new lines or using arrow keys for nav in insert mode.
        let (row, col) = self.cursor.location();
        self.buffer.insert_char_at(row, col, c);
        self.cursor.move_right(&self.buffer);
    }

    fn apply(&self, action: Action){
        // Empty just for compililation sake
        return;
    }
}
