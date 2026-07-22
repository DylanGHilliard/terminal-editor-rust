use crate::cursor::Position;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Viewport {
    pub width: usize,
    pub height: usize,
    pub offset_x: usize,
    pub offset_y: usize,
}

impl Viewport {
    pub const fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            offset_x: 0,
            offset_y: 0,
        }
    }

    pub fn set_size(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
    }

    pub fn ensure_visible(&mut self, position: Position) {
        self.offset_x = visible_offset(self.offset_x, self.width, position.column);
        self.offset_y = visible_offset(self.offset_y, self.height, position.line);
    }

    pub fn scroll_vertical(&mut self, lines: isize, line_count: usize) {
        let max_offset = line_count.saturating_sub(self.height.max(1));
        if lines < 0 {
            self.offset_y = self.offset_y.saturating_sub(lines.unsigned_abs());
        } else {
            self.offset_y = self.offset_y.saturating_add(lines as usize).min(max_offset);
        }
    }
}

fn visible_offset(current: usize, size: usize, target: usize) -> usize {
    if target < current {
        target
    } else if size == 0 {
        current
    } else if target >= current.saturating_add(size) {
        target + 1 - size
    } else {
        current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrolls_only_when_the_cursor_leaves_the_view() {
        let mut viewport = Viewport::new(5, 3);

        viewport.ensure_visible(Position::new(3, 5));
        assert_eq!((viewport.offset_x, viewport.offset_y), (1, 1));

        viewport.ensure_visible(Position::new(1, 1));
        assert_eq!((viewport.offset_x, viewport.offset_y), (1, 1));

        viewport.ensure_visible(Position::new(0, 0));
        assert_eq!((viewport.offset_x, viewport.offset_y), (0, 0));
    }

    #[test]
    fn wheel_scrolling_stays_inside_the_document() {
        let mut viewport = Viewport::new(20, 5);

        viewport.scroll_vertical(3, 20);
        assert_eq!(viewport.offset_y, 3);

        viewport.scroll_vertical(100, 20);
        assert_eq!(viewport.offset_y, 15);

        viewport.scroll_vertical(-100, 20);
        assert_eq!(viewport.offset_y, 0);
    }
}
