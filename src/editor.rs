use std::io;
use std::path::Path;
use std::time::Duration;

use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind,
};

use crate::clipboard::{Clipboard, MemoryClipboard};
use crate::cursor::Position;
use crate::document::{Direction, Document};
use crate::terminal::{Terminal, TerminalSession, line_number_gutter_width};

#[derive(Clone, Debug, Eq, PartialEq)]
enum EditorAction {
    Quit,
    Move(Direction, bool),
    Insert(String),
    Newline,
    Backspace,
    Delete,
    Undo,
    Redo,
    Copy,
    Cut,
    Paste,
    SelectAll,
    ClearSelection,
    None,
}

pub struct Editor {
    documents: Vec<Document>,
    current_document_index: usize,
    terminal: Terminal,
    clipboard: Box<dyn Clipboard>,
}

impl Editor {
    pub fn new() -> io::Result<Self> {
        Self::with_clipboard(Box::<MemoryClipboard>::default())
    }

    pub fn with_clipboard(clipboard: Box<dyn Clipboard>) -> io::Result<Self> {
        let terminal = Terminal::new()?;
        let mut document = Document::default();
        let (width, height) = content_size(terminal.size(), document.buffer().line_count());
        document.set_viewport_size(width, height);
        Ok(Self {
            documents: vec![document],
            current_document_index: 0,
            terminal,
            clipboard,
        })
    }

    pub fn open_file(&mut self, path: impl AsRef<Path>) -> io::Result<()> {
        let mut document = Document::open(path)?;
        let (width, height) = content_size(self.terminal.size(), document.buffer().line_count());
        document.set_viewport_size(width, height);

        // Replace the untouched startup buffer; subsequent opens become independent tabs.
        if self.documents.len() == 1
            && self.documents[0].buffer().path().is_none()
            && !self.documents[0].is_modified()
        {
            self.documents[0] = document;
            self.current_document_index = 0;
        } else {
            self.documents.push(document);
            self.current_document_index = self.documents.len() - 1;
        }
        Ok(())
    }

    pub fn run(&mut self) -> io::Result<()> {
        let _session = TerminalSession::enter()?;
        self.sync_current_viewport_size();
        self.terminal
            .draw(&self.documents[self.current_document_index])?;

        loop {
            if !event::poll(Duration::from_millis(50))? {
                continue;
            }

            let redraw = match event::read()? {
                Event::Key(key) if is_actionable_key(key) => {
                    if self.handle_action(action_for_key(key))? {
                        return Ok(());
                    }
                    true
                }
                Event::Paste(text) => {
                    self.current_document_mut().insert_text(&text);
                    true
                }
                Event::Mouse(mouse) => self.handle_mouse(mouse),
                Event::Resize(width, height) => {
                    self.terminal.resize(width, height);
                    true
                }
                _ => false,
            };

            if !redraw {
                continue;
            }

            self.sync_current_viewport_size();
            self.terminal
                .draw(&self.documents[self.current_document_index])?;
        }
    }

    fn handle_action(&mut self, action: EditorAction) -> io::Result<bool> {
        match action {
            EditorAction::Quit => return Ok(true),
            EditorAction::Move(direction, selecting) => {
                self.current_document_mut()
                    .move_cursor(direction, selecting);
            }
            EditorAction::Insert(text) => self.current_document_mut().insert_text(&text),
            EditorAction::Newline => self.current_document_mut().insert_newline(),
            EditorAction::Backspace => self.current_document_mut().backspace(),
            EditorAction::Delete => self.current_document_mut().delete(),
            EditorAction::Undo => self.current_document_mut().undo(),
            EditorAction::Redo => self.current_document_mut().redo(),
            EditorAction::Copy => {
                if let Some(text) = self.current_document().selected_text() {
                    self.clipboard.write(&text)?;
                }
            }
            EditorAction::Cut => {
                if let Some(text) = self.current_document_mut().cut_selection() {
                    self.clipboard.write(&text)?;
                }
            }
            EditorAction::Paste => {
                let text = self.clipboard.read()?;
                self.current_document_mut().insert_text(&text);
            }
            EditorAction::SelectAll => self.current_document_mut().select_all(),
            EditorAction::ClearSelection => self.current_document_mut().clear_selection(),
            EditorAction::None => {}
        }
        Ok(false)
    }

    fn sync_current_viewport_size(&mut self) {
        let line_count = self.current_document().buffer().line_count();
        let (width, height) = content_size(self.terminal.size(), line_count);
        self.current_document_mut().set_viewport_size(width, height);
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) -> bool {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(position) =
                    position_for_mouse(self.current_document(), mouse.column, mouse.row)
                {
                    let extend = mouse.modifiers.contains(KeyModifiers::SHIFT);
                    self.current_document_mut().set_cursor(position, extend);
                    true
                } else {
                    false
                }
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                if let Some(position) =
                    position_for_mouse(self.current_document(), mouse.column, mouse.row)
                {
                    self.current_document_mut().set_cursor(position, true);
                    true
                } else {
                    false
                }
            }
            MouseEventKind::ScrollUp => {
                self.current_document_mut().scroll_lines(-3);
                true
            }
            MouseEventKind::ScrollDown => {
                self.current_document_mut().scroll_lines(3);
                true
            }
            _ => false,
        }
    }

    fn current_document(&self) -> &Document {
        &self.documents[self.current_document_index]
    }

    fn current_document_mut(&mut self) -> &mut Document {
        &mut self.documents[self.current_document_index]
    }
}

fn content_size((width, height): (u16, u16), line_count: usize) -> (usize, usize) {
    (
        (width as usize).saturating_sub(line_number_gutter_width(line_count)),
        height.saturating_sub(1) as usize,
    )
}

fn position_for_mouse(document: &Document, column: u16, row: u16) -> Option<Position> {
    let viewport = document.viewport();
    if viewport.width == 0 || row as usize >= viewport.height {
        return None;
    }

    let gutter_width = line_number_gutter_width(document.buffer().line_count());
    let text_column = (column as usize).saturating_sub(gutter_width);
    Some(document.buffer().clamp_position(Position::new(
        viewport.offset_y + row as usize,
        viewport.offset_x + text_column,
    )))
}

fn is_actionable_key(key: KeyEvent) -> bool {
    matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat)
}

fn action_for_key(key: KeyEvent) -> EditorAction {
    let control = key.modifiers.contains(KeyModifiers::CONTROL);
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);
    let alt = key.modifiers.contains(KeyModifiers::ALT);

    if control && let KeyCode::Char(character) = key.code {
        return match character.to_ascii_lowercase() {
            'q' => EditorAction::Quit,
            'z' if shift => EditorAction::Redo,
            'z' => EditorAction::Undo,
            'y' => EditorAction::Redo,
            'c' => EditorAction::Copy,
            'x' => EditorAction::Cut,
            'v' => EditorAction::Paste,
            'a' => EditorAction::SelectAll,
            _ => EditorAction::None,
        };
    }

    match key.code {
        KeyCode::Up => EditorAction::Move(Direction::Up, shift),
        KeyCode::Down => EditorAction::Move(Direction::Down, shift),
        KeyCode::Left => EditorAction::Move(Direction::Left, shift),
        KeyCode::Right => EditorAction::Move(Direction::Right, shift),
        KeyCode::Home => EditorAction::Move(Direction::Home, shift),
        KeyCode::End => EditorAction::Move(Direction::End, shift),
        KeyCode::Backspace => EditorAction::Backspace,
        KeyCode::Delete => EditorAction::Delete,
        KeyCode::Enter => EditorAction::Newline,
        KeyCode::Tab => EditorAction::Insert("\t".to_owned()),
        KeyCode::Esc => EditorAction::ClearSelection,
        KeyCode::Char(character) if !control && !alt => EditorAction::Insert(character.to_string()),
        _ => EditorAction::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_q_is_text_and_control_q_quits() {
        let plain_q = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        let control_q = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);

        assert_eq!(
            action_for_key(plain_q),
            EditorAction::Insert("q".to_owned())
        );
        assert_eq!(action_for_key(control_q), EditorAction::Quit);
    }

    #[test]
    fn selection_and_history_shortcuts_map_to_actions() {
        let shift_left = KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT);
        let undo = KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL);
        let redo = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::CONTROL);

        assert_eq!(
            action_for_key(shift_left),
            EditorAction::Move(Direction::Left, true)
        );
        assert_eq!(action_for_key(undo), EditorAction::Undo);
        assert_eq!(action_for_key(redo), EditorAction::Redo);
    }

    #[test]
    fn content_width_reserves_space_for_line_numbers() {
        assert_eq!(content_size((80, 24), 9), (77, 23));
        assert_eq!(content_size((80, 24), 10), (76, 23));
        assert_eq!(content_size((2, 1), 100), (0, 0));
    }

    #[test]
    fn mouse_coordinates_account_for_the_gutter_and_viewport() {
        let mut document = Document::new(crate::buffer::Buffer::from_text(
            None,
            "zero\none\ntwo\nthree",
        ));
        document.set_viewport_size(20, 2);
        document.scroll_lines(1);

        assert_eq!(
            position_for_mouse(&document, 0, 0),
            Some(Position::new(1, 0))
        );
        assert_eq!(
            position_for_mouse(&document, 5, 1),
            Some(Position::new(2, 2))
        );
        assert_eq!(position_for_mouse(&document, 5, 2), None);
    }
}
