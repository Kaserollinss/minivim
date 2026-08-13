mod editor;
mod input;
mod terminal;

use editor::Editor;

fn main() {
    let mut editor = Editor::default();

    editor.run();
}
