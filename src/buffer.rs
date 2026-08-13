use std::path::{Path, PathBuf};
use std::{fs, io};

pub struct Buffer {
    lines: Vec<String>,
    filename: Option<PathBuf>,
    modified: bool,

}

impl Buffer {
    fn empty() -> Self {
        Buffer {
            lines: vec![String::new()],
            filename: None,
            modified: false
        }
    }

    fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
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
}
