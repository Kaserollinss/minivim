mod buffer;
mod cursor;
mod editor;
mod input;
mod pos;
mod terminal;
mod view;
mod walker;

use std::path::PathBuf;

use editor::Editor;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let path: Option<PathBuf> = args.get(1).map(PathBuf::from);
    if let Some(path) = path {
        let editor = Editor::from_file(path);
        editor.unwrap().run();
    }
    if args.len() == 1 {
        let mut editor = Editor::default();
        editor.run()
    } //let mut editor = Editor::default();
}
