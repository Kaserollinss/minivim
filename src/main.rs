mod buffer;
mod cursor;
mod editor;
mod input;
mod terminal;
mod view;

use std::path;

use editor::Editor;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let path:Option<path::PathBuf> = args.get(1).map(|s| path::PathBuf::from(s));
    let mut editor = Editor::default();

    editor.run();
}
