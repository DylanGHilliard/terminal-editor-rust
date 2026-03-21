
mod buffer;
mod editor;
mod viewport;
mod terminal;

fn main() {
    let mut editor = editor::Editor::new();
    editor.main_loop();
}