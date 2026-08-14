use std::path::{Path, PathBuf};
use std::{fs, io};

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
            modified: false
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path = path.as_ref();
        let contents = fs::read_to_string(path)?; // '?' forces a return Err() if read_to_string breaks
        let lines: Vec<String> = contents.lines().map(String::from).collect();
        let lines = if lines.is_empty() { vec![String::new()] } else { lines };

        Ok(Buffer {
            lines,
            filename: Some(path.to_path_buf()),
            modified: false,
        })
    }

    pub fn insert_char_at(&self, row: usize, col: usize, c: char){
        let temp_line = self.lines.get_mut(row);
        self.lines.insert()
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
}
