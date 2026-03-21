
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


}