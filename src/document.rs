use std::io;
use std::path::Path;

use crate::buffer::Buffer;
use crate::cursor::{Cursor, CursorState, Position, TextRange};
use crate::history::EditHistory;
use crate::viewport::Viewport;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
}

#[derive(Debug)]
pub struct Document {
    buffer: Buffer,
    cursor: Cursor,
    viewport: Viewport,
    history: EditHistory,
}

impl Default for Document {
    fn default() -> Self {
        Self::new(Buffer::new())
    }
}

impl Document {
    pub fn new(buffer: Buffer) -> Self {
        Self {
            buffer,
            cursor: Cursor::new(),
            viewport: Viewport::default(),
            history: EditHistory::default(),
        }
    }

    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        Ok(Self::new(Buffer::open(path)?))
    }

    pub const fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub const fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    pub const fn viewport(&self) -> &Viewport {
        &self.viewport
    }

    pub fn set_viewport_size(&mut self, width: usize, height: usize) {
        if self.viewport.width == width && self.viewport.height == height {
            return;
        }
        self.viewport.set_size(width, height);
        self.reveal_cursor();
    }

    pub fn set_cursor(&mut self, position: Position, extend_selection: bool) {
        let position = self.buffer.clamp_position(position);
        if extend_selection {
            self.cursor.start_selection();
        } else {
            self.cursor.clear_selection();
        }
        self.cursor.set_position(position, true);
        self.reveal_cursor();
    }

    pub fn scroll_lines(&mut self, lines: isize) {
        self.viewport
            .scroll_vertical(lines, self.buffer.line_count());
    }

    pub fn is_modified(&self) -> bool {
        self.history.is_modified()
    }

    pub fn insert_text(&mut self, text: &str) {
        let range = self
            .cursor
            .selection()
            .unwrap_or_else(|| TextRange::empty(self.cursor.position()));
        self.apply_edit(range, text);
    }

    pub fn insert_newline(&mut self) {
        self.insert_text("\n");
    }

    pub fn backspace(&mut self) {
        if let Some(selection) = self.cursor.selection() {
            self.apply_edit(selection, "");
            return;
        }

        let position = self.cursor.position();
        let start = if position.column > 0 {
            Position::new(position.line, position.column - 1)
        } else if position.line > 0 {
            Position::new(position.line - 1, self.buffer.line_len(position.line - 1))
        } else {
            return;
        };
        self.apply_edit(TextRange::new(start, position), "");
    }

    pub fn delete(&mut self) {
        if let Some(selection) = self.cursor.selection() {
            self.apply_edit(selection, "");
            return;
        }

        let position = self.cursor.position();
        let end = if position.column < self.buffer.line_len(position.line) {
            Position::new(position.line, position.column + 1)
        } else if position.line + 1 < self.buffer.line_count() {
            Position::new(position.line + 1, 0)
        } else {
            return;
        };
        self.apply_edit(TextRange::new(position, end), "");
    }

    pub fn selected_text(&self) -> Option<String> {
        self.cursor
            .selection()
            .map(|range| self.buffer.text_in_range(range))
    }

    pub fn cut_selection(&mut self) -> Option<String> {
        let range = self.cursor.selection()?;
        let text = self.buffer.text_in_range(range);
        self.apply_edit(range, "");
        Some(text)
    }

    pub fn select_all(&mut self) {
        let end_line = self.buffer.line_count() - 1;
        let end = Position::new(end_line, self.buffer.line_len(end_line));
        self.cursor.collapse_to(Position::new(0, 0));
        self.cursor.start_selection();
        self.cursor.set_position(end, true);
        self.reveal_cursor();
    }

    pub fn clear_selection(&mut self) {
        self.cursor.clear_selection();
    }

    pub fn undo(&mut self) {
        if let Some(cursor) = self.history.undo(&mut self.buffer) {
            self.cursor.restore(cursor);
            self.reveal_cursor();
        }
    }

    pub fn redo(&mut self) {
        if let Some(cursor) = self.history.redo(&mut self.buffer) {
            self.cursor.restore(cursor);
            self.reveal_cursor();
        }
    }

    pub fn move_cursor(&mut self, direction: Direction, extend_selection: bool) {
        if extend_selection {
            self.cursor.start_selection();
        } else if let Some(selection) = self.cursor.selection() {
            let collapse_position = match direction {
                Direction::Left | Direction::Home | Direction::Up => selection.start,
                Direction::Right | Direction::End | Direction::Down => selection.end,
            };
            self.cursor.collapse_to(collapse_position);
            self.reveal_cursor();
            return;
        } else {
            self.cursor.clear_selection();
        }

        let position = self.cursor.position();
        let preferred_column = self.cursor.preferred_column();
        let (new_position, update_preferred_column) = match direction {
            Direction::Left if position.column > 0 => {
                (Position::new(position.line, position.column - 1), true)
            }
            Direction::Left if position.line > 0 => {
                let line = position.line - 1;
                (Position::new(line, self.buffer.line_len(line)), true)
            }
            Direction::Right if position.column < self.buffer.line_len(position.line) => {
                (Position::new(position.line, position.column + 1), true)
            }
            Direction::Right if position.line + 1 < self.buffer.line_count() => {
                (Position::new(position.line + 1, 0), true)
            }
            Direction::Up if position.line > 0 => {
                let line = position.line - 1;
                (
                    Position::new(line, preferred_column.min(self.buffer.line_len(line))),
                    false,
                )
            }
            Direction::Down if position.line + 1 < self.buffer.line_count() => {
                let line = position.line + 1;
                (
                    Position::new(line, preferred_column.min(self.buffer.line_len(line))),
                    false,
                )
            }
            Direction::Home => (Position::new(position.line, 0), true),
            Direction::End => (
                Position::new(position.line, self.buffer.line_len(position.line)),
                true,
            ),
            _ => (position, false),
        };

        self.cursor
            .set_position(new_position, update_preferred_column);
        self.reveal_cursor();
    }

    fn apply_edit(&mut self, range: TextRange, replacement: &str) {
        let cursor_before = self.cursor.snapshot();
        let change = self.buffer.replace_range(range, replacement);
        let cursor_after = CursorState::collapsed(change.inserted_range().end);
        self.cursor.restore(cursor_after);
        self.history.record(change, cursor_before, cursor_after);
        self.reveal_cursor();
    }

    fn reveal_cursor(&mut self) {
        self.viewport.ensure_visible(self.cursor.position());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn document(text: &str) -> Document {
        Document::new(Buffer::from_text(None, text))
    }

    #[test]
    fn edit_actions_share_undo_and_redo_history() {
        let mut document = document("one\ntwo");
        document.move_cursor(Direction::End, false);
        document.insert_text("!");
        document.insert_newline();
        document.insert_text("next");
        document.backspace();

        assert_eq!(document.buffer().text(), "one!\nnex\ntwo");

        document.undo();
        assert_eq!(document.buffer().text(), "one!\nnext\ntwo");
        document.undo();
        assert_eq!(document.buffer().text(), "one!\n\ntwo");
        document.redo();
        assert_eq!(document.buffer().text(), "one!\nnext\ntwo");
    }

    #[test]
    fn selection_can_be_copied_cut_and_pasted() {
        let mut document = document("alpha beta");
        for _ in 0..5 {
            document.move_cursor(Direction::Right, true);
        }

        assert_eq!(document.selected_text().as_deref(), Some("alpha"));
        assert_eq!(document.cut_selection().as_deref(), Some("alpha"));
        assert_eq!(document.buffer().text(), " beta");

        document.move_cursor(Direction::End, false);
        document.insert_text("alpha");
        assert_eq!(document.buffer().text(), " betaalpha");
    }

    #[test]
    fn a_new_edit_clears_the_redo_branch() {
        let mut document = document("");
        document.insert_text("a");
        document.undo();
        document.insert_text("b");
        document.redo();

        assert_eq!(document.buffer().text(), "b");
    }

    #[test]
    fn vertical_movement_remembers_the_preferred_column() {
        let mut document = document("12345\n1\n12345");
        document.move_cursor(Direction::End, false);
        document.move_cursor(Direction::Down, false);
        assert_eq!(document.cursor().position(), Position::new(1, 1));

        document.move_cursor(Direction::Down, false);
        assert_eq!(document.cursor().position(), Position::new(2, 5));
    }

    #[test]
    fn mouse_style_cursor_placement_clamps_and_extends_selection() {
        let mut document = document("first\nsecond");

        document.set_cursor(Position::new(0, 2), false);
        document.set_cursor(Position::new(50, 50), true);

        assert_eq!(document.cursor().position(), Position::new(1, 6));
        assert_eq!(document.selected_text().as_deref(), Some("rst\nsecond"));
    }
}
