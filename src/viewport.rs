
pub struct Viewport {
    pub width: usize,
    pub height: usize,
    pub offset_x: usize,
    pub offset_y: usize,

}

impl Viewport {
    pub fn new(width: usize, height: usize, offset_x: usize, offset_y: usize) -> Self {
        Viewport {
            width,
            height,
            offset_x,
            offset_y,
        }
    }
    pub fn set_size(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.offset_x = self.offset_x.clamp(0, width - 1);
        self.offset_y = self.offset_y.clamp(0, height - 1);
    }

    pub fn set_offset(&mut self, offset_x: usize, offset_y: usize) {
        self.offset_x = offset_x.clamp(0, self.width.saturating_sub(1));
        self.offset_y = offset_y.clamp(0, self.height.saturating_sub(1));
    }

    pub fn get_offset(&self) -> (usize, usize) {
        (self.offset_x, self.offset_y)
    }

    pub fn scroll_down(&mut self, lines: usize) {

        self.offset_y =  self.offset_y.saturating_add(lines).clamp(0, self.height.saturating_sub(1));
    }

    pub fn scroll_up(&mut self, lines: usize) {
        self.offset_y = self.offset_y.saturating_sub(lines).clamp(0, self.height.saturating_sub(1));
    }

    pub fn scroll_left(&mut self, cols: usize) {
        self.offset_x = self.offset_x.saturating_sub(cols).clamp(0, self.width.saturating_sub(1));
    }

    pub fn scroll_right(&mut self, cols: usize) {
        self.offset_x = self.offset_x.saturating_add(cols).clamp(0, self.width.saturating_sub(1));
    }
}