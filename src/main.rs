mod buffer;
mod cursor;
mod editor;
mod input;
mod terminal;
mod view;

use std::path::PathBuf;

use editor::Editor;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let path:Option<PathBuf> = args.get(1).map(|s| PathBuf::from(s));
    if let Some(path) = path {      
        let mut editor = Editor::from_file(path);
        editor.unwrap().run();
    }
    if args.len() == 1 {
        let mut editor = Editor::default();
        editor.run()
    }    //let mut editor = Editor::default();
}
