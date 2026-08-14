use crossterm::event::{read, Event, Event::Key, KeyCode, KeyCode::Char, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType};
use std::io::stdout;

pub struct Editor {
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
            should_quit: false,
            mode: Mode::Normal,
        }
    }
    pub fn run(&mut self) {
        Self::initialize().unwrap();
        let result = self.repl();
        Self::terminate().unwrap();
        result.unwrap();
    }

    fn initialize() -> Result<(), std::io::Error> {
        enable_raw_mode()?;
        Self::clear_screen()
    }
    fn terminate() -> Result<(), std::io::Error> {
        disable_raw_mode()
    }
    fn clear_screen() -> Result<(), std::io::Error> {
        let mut stdout = stdout();
        execute!(stdout, Clear(ClearType::All))
    }

    fn repl(&mut self) -> Result<(), std::io::Error> {
        loop {
            let event = read()?;
            match self.mode {
                Mode::Normal => self.evaluate_normal_mode(&event),
                Mode::Insert => self.evaluate_insert_mode(&event),
                Mode::Visual => self.evaluate_visual_mode(&event),
                Mode::Command => (), // TODO: not implemented yet
            }
        
            self.evaluate_event(&event);
            self.refresh_screen()?;
            if self.should_quit {
                break;
            }
        }
        Ok(())
    }
    fn evaluate_event(&mut self, event: &Event) {
        if let Key(KeyEvent {
            code, modifiers, ..
        }) = event
        {
            match code {
                Char('q') if *modifiers == KeyModifiers::CONTROL => {
                    self.should_quit = true;
                }
                _ => (),
            }
        }
    }
    // TODO: placeholder so the crate compiles — only handles leaving visual mode.
    fn evaluate_visual_mode(&mut self, event: &Event) {
        if let Key(KeyEvent { code, .. }) = event {
            if matches!(code, KeyCode::Esc) {
                self.mode = Mode::Normal;
            }
        }
    }
    fn evaluate_normal_mode(&mut self, event: &Event) {
        if let Key(KeyEvent {
            code, modifiers, ..
        }) = event
        {
            match code {
                Char('i') => {
                    self.mode = Mode::Insert;
                },
                Char('v') => {
                    self.mode = Mode::Visual;
                },
                _ => (),
            }
        }
    }
    fn evaluate_insert_mode(&mut self, event: &Event) {
        if let Key(KeyEvent {   
            code, modifiers, ..
        }) = event
        {
            match code {
                Char('q') if *modifiers == KeyModifiers::CONTROL => {
                    self.mode = Mode::Normal;
                }
                _ => (),
            }
        }
    }
    fn refresh_screen(&self) -> Result<(), std::io::Error> {
        if self.should_quit {
            Self::clear_screen()?;
            print!("Goodbye.\r\n");
        }
        Ok(())
    }
}
