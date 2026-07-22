use std::io::{self, Stdout, Write, stdout};

use crossterm::{
    QueueableCommand, cursor,
    event::{DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture},
    execute,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{
        self as ct, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};

use crate::cursor::TextRange;
use crate::document::Document;

pub struct Terminal {
    stdout: Stdout,
    size: (u16, u16),
}

impl Terminal {
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            stdout: stdout(),
            size: ct::size()?,
        })
    }

    pub const fn size(&self) -> (u16, u16) {
        self.size
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.size = (width, height);
    }

    pub fn draw(&mut self, document: &Document) -> io::Result<()> {
        self.stdout.queue(cursor::MoveTo(0, 0))?;
        self.stdout.queue(ct::Clear(ClearType::All))?;

        let viewport = document.viewport();
        let buffer = document.buffer();
        let selection = document.cursor().selection();
        let line_number_width = decimal_digit_count(buffer.line_count());
        let gutter_width = line_number_gutter_width(buffer.line_count());

        for (screen_y, line) in buffer
            .lines()
            .iter()
            .skip(viewport.offset_y)
            .take(viewport.height)
            .enumerate()
        {
            let line_index = viewport.offset_y + screen_y;
            self.stdout.queue(cursor::MoveTo(0, screen_y as u16))?;
            self.draw_line_number(line_index + 1, line_number_width)?;
            self.draw_line(
                line,
                line_index,
                viewport.offset_x,
                viewport.width,
                selection,
            )?;
        }

        self.draw_status(document)?;

        let position = document.cursor().position();
        let cursor_is_visible = viewport.width > 0
            && viewport.height > 0
            && position.column >= viewport.offset_x
            && position.column < viewport.offset_x.saturating_add(viewport.width)
            && position.line >= viewport.offset_y
            && position.line < viewport.offset_y.saturating_add(viewport.height);
        if cursor_is_visible {
            let screen_x = gutter_width + position.column.saturating_sub(viewport.offset_x);
            let screen_y = position.line.saturating_sub(viewport.offset_y) as u16;
            self.stdout
                .queue(cursor::MoveTo(screen_x as u16, screen_y))?;
            self.stdout.queue(cursor::Show)?;
        } else {
            self.stdout.queue(cursor::Hide)?;
        }
        self.stdout.flush()
    }

    fn draw_line_number(&mut self, line_number: usize, width: usize) -> io::Result<()> {
        let gutter = format!("{line_number:>width$} │");
        let visible_gutter: String = gutter.chars().take(self.size.0 as usize).collect();

        self.stdout.queue(SetForegroundColor(Color::DarkGrey))?;
        self.stdout.queue(Print(visible_gutter))?;
        self.stdout.queue(ResetColor)?;
        Ok(())
    }

    fn draw_line(
        &mut self,
        line: &str,
        line_index: usize,
        offset_x: usize,
        width: usize,
        selection: Option<TextRange>,
    ) -> io::Result<()> {
        let view_end = offset_x.saturating_add(width);
        let Some((selection_start, selection_end)) =
            selection.and_then(|range| selected_columns(range, line_index, line.chars().count()))
        else {
            return self
                .stdout
                .queue(Print(characters_between(line, offset_x, view_end)))
                .map(|_| ());
        };

        let highlighted_start = selection_start.max(offset_x).min(view_end);
        let highlighted_end = selection_end.max(offset_x).min(view_end);

        self.stdout
            .queue(Print(characters_between(line, offset_x, highlighted_start)))?;
        if highlighted_start < highlighted_end {
            self.stdout.queue(SetAttribute(Attribute::Reverse))?;
            self.stdout.queue(Print(characters_between(
                line,
                highlighted_start,
                highlighted_end,
            )))?;
            self.stdout.queue(SetAttribute(Attribute::NoReverse))?;
        }
        self.stdout
            .queue(Print(characters_between(line, highlighted_end, view_end)))?;
        Ok(())
    }

    fn draw_status(&mut self, document: &Document) -> io::Result<()> {
        let (width, height) = self.size;
        if width == 0 || height == 0 {
            return Ok(());
        }

        let modified = if document.is_modified() { " [+]" } else { "" };
        let status = format!(
            " {}{}  Ctrl-Q Quit | Ctrl-Z/Y Undo/Redo | Ctrl-C/X/V Clipboard ",
            document.buffer().display_name(),
            modified
        );
        let mut status: String = status.chars().take(width as usize).collect();
        status.extend(std::iter::repeat_n(
            ' ',
            width as usize - status.chars().count(),
        ));

        self.stdout.queue(cursor::MoveTo(0, height - 1))?;
        self.stdout.queue(SetAttribute(Attribute::Reverse))?;
        self.stdout.queue(Print(status))?;
        self.stdout.queue(SetAttribute(Attribute::NoReverse))?;
        Ok(())
    }
}

pub(crate) fn line_number_gutter_width(line_count: usize) -> usize {
    decimal_digit_count(line_count) + 2
}

fn decimal_digit_count(number: usize) -> usize {
    number.max(1).ilog10() as usize + 1
}

pub struct TerminalSession;

impl TerminalSession {
    pub fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        if let Err(error) = execute!(
            stdout(),
            EnterAlternateScreen,
            EnableBracketedPaste,
            EnableMouseCapture
        ) {
            let _ = execute!(
                stdout(),
                DisableMouseCapture,
                DisableBracketedPaste,
                LeaveAlternateScreen
            );
            let _ = disable_raw_mode();
            return Err(error);
        }
        Ok(Self)
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = execute!(
            stdout(),
            DisableMouseCapture,
            DisableBracketedPaste,
            LeaveAlternateScreen
        );
        let _ = disable_raw_mode();
    }
}

fn characters_between(text: &str, start: usize, end: usize) -> String {
    text.chars()
        .skip(start)
        .take(end.saturating_sub(start))
        .collect()
}

fn selected_columns(
    selection: TextRange,
    line: usize,
    line_length: usize,
) -> Option<(usize, usize)> {
    if line < selection.start.line || line > selection.end.line {
        return None;
    }

    let start = if line == selection.start.line {
        selection.start.column
    } else {
        0
    };
    let end = if line == selection.end.line {
        selection.end.column
    } else {
        line_length
    };
    (start < end).then_some((start, end))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gutter_grows_when_line_numbers_gain_a_digit() {
        assert_eq!(line_number_gutter_width(1), 3);
        assert_eq!(line_number_gutter_width(9), 3);
        assert_eq!(line_number_gutter_width(10), 4);
        assert_eq!(line_number_gutter_width(100), 5);
    }
}
