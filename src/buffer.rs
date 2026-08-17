use std::path::{Path, PathBuf};
use std::{fs, io};

use crate::pos::Pos;

pub struct Buffer {
    lines: Vec<String>,
    filename: Option<PathBuf>,
    modified: bool,
}

impl Buffer {
    pub fn empty() -> Self {
        Buffer {
            lines: vec![String::new()],
            filename: None,
            modified: false,
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path = path.as_ref();
        let contents = fs::read_to_string(path)?; // '?' forces a return Err() if read_to_string breaks
        let lines: Vec<String> = contents.lines().map(String::from).collect();
        let lines = if lines.is_empty() {
            vec![String::new()]
        } else {
            lines
        };

        Ok(Buffer {
            lines,
            filename: Some(path.to_path_buf()),
            modified: false,
        })
    }

    /// Build a buffer from known lines, no file. Test seam.
    #[cfg(test)]
    pub(crate) fn from_lines(lines: &[&str]) -> Self {
        let lines = if lines.is_empty() {
            vec![String::new()]
        } else {
            lines.iter().map(|l| l.to_string()).collect()
        };
        Buffer {
            lines,
            filename: None,
            modified: false,
        }
    }

    pub fn insert_char_at(&mut self, row: usize, col: usize, c: char) {
        let line: Option<&mut String> = self.lines.get_mut(row);
        match line {
            Some(line) => line.insert(col, c),
            None => (),
        }
    }

    pub fn remove_char_at(&mut self, pos: Pos) {
        let line = self.lines.get_mut(pos.row);
        match line {
            Some(line) => {
                line.remove(pos.col);
            }
            None => (),
        }
    }

    pub fn new_line_at(&mut self, row: usize) {
        if self.len() > row {
            self.lines.insert(row, String::new())
        }
    }

    // pub fn new_line_below(&mut self, row: usize) {
    //     if self.len() > row {
    //         self.lines.insert(row + 1, String::new())
    //     }
    // }

    pub fn char_at(&self, pos: Pos) -> Option<char> {
        let line = self.line(pos.row)?; // None if the row doesn't exist
        let len = line.chars().count();
        if pos.col < len {
            line.chars().nth(pos.col)
        } else if pos.col == len {
            Some('\n') // the end-of-line slot, standing in for the stripped newline
        } else {
            None // past the end of the line
        }
    }

    pub fn next_pos(&self, pos: Pos) -> Option<Pos> {
        if pos.col < self.line_len(pos.row) {
            // still within this line - col+1 may be the '\n' slot
            Some(Pos {
                row: pos.row,
                col: pos.col + 1,
            })
        } else if self.has_next_line(pos.row) {
            // at the '\n' slot, hop to the start of the next line
            Some(Pos {
                row: pos.row + 1,
                col: 0,
            })
        } else {
            None // last line's slot - end of buffer
        }
    }

    pub fn prev_pos(&self, pos: Pos) -> Option<Pos> {
        if pos.col > 0 {
            Some(Pos {
                row: pos.row,
                col: pos.col - 1,
            })
        } else if pos.row > 0 {
            // at column 0, step up to the previous line's '\n' slot
            Some(Pos {
                row: pos.row - 1,
                col: self.line_len(pos.row - 1),
            })
        } else {
            None // start of buffer
        }
    }

    // Basic Buffer props

    pub fn line(&self, idx: usize) -> Option<&str> {
        self.lines.get(idx).map(String::as_str)
    }

    pub fn line_len(&self, idx: usize) -> usize {
        self.lines.get(idx).map_or(0, |l| l.chars().count())
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.len() == 1 && self.lines[0].is_empty()
    }

    pub fn filename(&self) -> Option<&Path> {
        self.filename.as_deref()
    }

    pub fn is_modified(&self) -> bool {
        self.modified
    }

    pub fn has_next_line(&self, row: usize) -> bool {
        row + 1 < self.len()
    }
}
