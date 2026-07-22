use crate::buffer::{Buffer, TextChange};
use crate::cursor::CursorState;

#[derive(Clone, Debug, Eq, PartialEq)]
struct HistoryEntry {
    change: TextChange,
    cursor_before: CursorState,
    cursor_after: CursorState,
    revision_before: u64,
    revision_after: u64,
}

#[derive(Clone, Debug)]
pub struct EditHistory {
    undo: Vec<HistoryEntry>,
    redo: Vec<HistoryEntry>,
    current_revision: u64,
    saved_revision: u64,
    next_revision: u64,
}

impl Default for EditHistory {
    fn default() -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            current_revision: 0,
            saved_revision: 0,
            next_revision: 1,
        }
    }
}

impl EditHistory {
    pub fn record(
        &mut self,
        change: TextChange,
        cursor_before: CursorState,
        cursor_after: CursorState,
    ) {
        if change.is_noop() {
            return;
        }

        let revision_after = self.next_revision;
        self.next_revision += 1;
        self.undo.push(HistoryEntry {
            change,
            cursor_before,
            cursor_after,
            revision_before: self.current_revision,
            revision_after,
        });
        self.redo.clear();
        self.current_revision = revision_after;
    }

    pub fn undo(&mut self, buffer: &mut Buffer) -> Option<CursorState> {
        let entry = self.undo.pop()?;
        let inverse = buffer.replace_range(entry.change.inserted_range(), &entry.change.removed);
        debug_assert_eq!(inverse.removed, entry.change.inserted);
        self.current_revision = entry.revision_before;
        let cursor = entry.cursor_before;
        self.redo.push(entry);
        Some(cursor)
    }

    pub fn redo(&mut self, buffer: &mut Buffer) -> Option<CursorState> {
        let entry = self.redo.pop()?;
        let reapplied = buffer.replace_range(entry.change.removed_range(), &entry.change.inserted);
        debug_assert_eq!(reapplied.removed, entry.change.removed);
        self.current_revision = entry.revision_after;
        let cursor = entry.cursor_after;
        self.undo.push(entry);
        Some(cursor)
    }

    pub fn is_modified(&self) -> bool {
        self.current_revision != self.saved_revision
    }

    pub fn mark_saved(&mut self) {
        self.saved_revision = self.current_revision;
    }
}
