use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::cursor::{Position, TextRange};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextChange {
    pub start: Position,
    pub removed: String,
    pub inserted: String,
}

impl TextChange {
    pub fn removed_range(&self) -> TextRange {
        TextRange::new(self.start, position_after(self.start, &self.removed))
    }

    pub fn inserted_range(&self) -> TextRange {
        TextRange::new(self.start, position_after(self.start, &self.inserted))
    }

    pub fn is_noop(&self) -> bool {
        self.removed == self.inserted
    }
}

#[derive(Clone, Debug)]
pub struct Buffer {
    lines: Vec<String>,
    path: Option<PathBuf>,
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

impl Buffer {
    pub fn new() -> Self {
        Self::from_text(None, "")
    }

    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();
        let text = fs::read_to_string(path)?;
        Ok(Self::from_text(Some(path.to_path_buf()), &text))
    }

    pub fn from_text(path: Option<PathBuf>, text: &str) -> Self {
        Self {
            // `split` deliberately preserves an empty final line when the file ends in a newline.
            lines: text
                .split('\n')
                .map(|line| line.strip_suffix('\r').unwrap_or(line).to_owned())
                .collect(),
            path,
        }
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn display_name(&self) -> String {
        self.path
            .as_deref()
            .and_then(Path::file_name)
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "[untitled]".to_owned())
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    pub fn line(&self, line: usize) -> Option<&str> {
        self.lines.get(line).map(String::as_str)
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Returns a line length in Unicode scalar values, matching cursor columns.
    pub fn line_len(&self, line: usize) -> usize {
        self.line(line).map_or(0, |line| line.chars().count())
    }

    pub fn text(&self) -> String {
        self.lines.join("\n")
    }

    pub fn clamp_position(&self, position: Position) -> Position {
        let line = position.line.min(self.line_count().saturating_sub(1));
        Position::new(line, position.column.min(self.line_len(line)))
    }

    pub fn text_in_range(&self, range: TextRange) -> String {
        let range = self.clamp_range(range);
        if range.is_empty() {
            return String::new();
        }

        if range.start.line == range.end.line {
            return char_slice(
                &self.lines[range.start.line],
                range.start.column,
                range.end.column,
            )
            .to_owned();
        }

        let mut text = String::new();
        text.push_str(char_slice_from(
            &self.lines[range.start.line],
            range.start.column,
        ));
        text.push('\n');

        for line in (range.start.line + 1)..range.end.line {
            text.push_str(&self.lines[line]);
            text.push('\n');
        }

        text.push_str(char_slice_to(&self.lines[range.end.line], range.end.column));
        text
    }

    /// Applies every edit through one range-replacement primitive.
    ///
    /// Insert, paste, delete, line split, line join, cut, undo, and redo can all be
    /// represented by this operation, which keeps future editing features composable.
    pub fn replace_range(&mut self, range: TextRange, replacement: &str) -> TextChange {
        let range = self.clamp_range(range);
        let removed = self.text_in_range(range);

        let start_line = &self.lines[range.start.line];
        let prefix = char_slice_to(start_line, range.start.column).to_owned();
        let end_line = &self.lines[range.end.line];
        let suffix = char_slice_from(end_line, range.end.column).to_owned();

        let mut replacement_lines = replacement.split('\n');
        let first = replacement_lines.next().unwrap_or_default();
        let remaining: Vec<&str> = replacement_lines.collect();
        let new_lines = if remaining.is_empty() {
            vec![format!("{prefix}{first}{suffix}")]
        } else {
            let mut lines = Vec::with_capacity(remaining.len() + 1);
            lines.push(format!("{prefix}{first}"));
            for middle in &remaining[..remaining.len() - 1] {
                lines.push((*middle).to_owned());
            }
            lines.push(format!("{}{suffix}", remaining[remaining.len() - 1]));
            lines
        };

        self.lines
            .splice(range.start.line..=range.end.line, new_lines);

        TextChange {
            start: range.start,
            removed,
            inserted: replacement.to_owned(),
        }
    }

    fn clamp_range(&self, range: TextRange) -> TextRange {
        let range = range.normalized();
        TextRange::new(
            self.clamp_position(range.start),
            self.clamp_position(range.end),
        )
        .normalized()
    }
}

pub fn position_after(start: Position, text: &str) -> Position {
    let mut parts = text.split('\n');
    let first = parts.next().unwrap_or_default();
    let mut line_count = 0;
    let mut final_column = first.chars().count();

    for part in parts {
        line_count += 1;
        final_column = part.chars().count();
    }

    if line_count == 0 {
        Position::new(start.line, start.column + final_column)
    } else {
        Position::new(start.line + line_count, final_column)
    }
}

fn byte_index(text: &str, column: usize) -> usize {
    text.char_indices()
        .nth(column)
        .map_or(text.len(), |(index, _)| index)
}

fn char_slice(text: &str, start: usize, end: usize) -> &str {
    &text[byte_index(text, start)..byte_index(text, end)]
}

fn char_slice_from(text: &str, start: usize) -> &str {
    &text[byte_index(text, start)..]
}

fn char_slice_to(text: &str, end: usize) -> &str {
    &text[..byte_index(text, end)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_buffer_always_has_one_editable_line() {
        let buffer = Buffer::new();

        assert_eq!(buffer.lines(), &[String::new()]);
        assert_eq!(
            buffer.clamp_position(Position::new(20, 20)),
            Position::new(0, 0)
        );
    }

    #[test]
    fn replaces_a_multiline_range() {
        let mut buffer = Buffer::from_text(None, "hello\nwide world\nlast");

        let change = buffer.replace_range(
            TextRange::new(Position::new(0, 3), Position::new(2, 2)),
            "p!\nnew",
        );

        assert_eq!(change.removed, "lo\nwide world\nla");
        assert_eq!(buffer.text(), "help!\nnewst");
        assert_eq!(change.inserted_range().end, Position::new(1, 3));
    }

    #[test]
    fn cursor_columns_are_unicode_safe() {
        let mut buffer = Buffer::from_text(None, "a🦀b");

        buffer.replace_range(TextRange::empty(Position::new(0, 2)), "é");
        buffer.replace_range(TextRange::new(Position::new(0, 1), Position::new(0, 2)), "");

        assert_eq!(buffer.text(), "aéb");
        assert_eq!(buffer.line_len(0), 3);
    }

    #[test]
    fn opens_windows_line_endings_without_rendering_carriage_returns() {
        let buffer = Buffer::from_text(None, "one\r\ntwo\r\n");

        assert_eq!(buffer.lines(), &["one", "two", ""]);
    }
}
