use std::error::Error;

use terminal_editor_rust::editor::Editor;

fn main() -> Result<(), Box<dyn Error>> {
    let mut editor = Editor::new()?;
    if let Some(path) = std::env::args_os().nth(1) {
        editor.open_file(path)?;
    }
    editor.run()?;
    Ok(())
}
