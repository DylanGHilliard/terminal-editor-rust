use std::io::Write;
use crossterm::{
    cursor,
    terminal as ct,
    terminal::ClearType,
    QueueableCommand,
    style::Print,
};

use crate::{buffer, viewport};
pub struct Terminal {
    stdout: std::io::Stdout,
    pub size: (u16, u16),
}
impl Terminal {
    pub fn new(stdout: std::io::Stdout) -> Terminal {
        Terminal { stdout, size: ct::size().unwrap() }
    }

    pub fn draw(&mut self, viewport: &viewport::Viewport, buffer: &buffer::Buffer, cursor_x: usize, cursor_y: usize) {
         
        self.stdout.queue(cursor::MoveTo(0, 0)).unwrap();
        self.stdout.queue(ct::Clear(ClearType::All)).unwrap();
        
        for (i, line) in buffer.data.iter().enumerate().skip(viewport.offset_y).take(self.size.1 as usize) {

            let  display_line = if line.len() > self.size.0 as usize {
                    &line[viewport.offset_x..self.size.0 as usize]
            } else if line.len() > viewport.offset_x 
                { &line[viewport.offset_x..] }
             else { "" };
            self.stdout.queue(cursor::MoveTo(0, (i - viewport.offset_y) as u16 )).unwrap();
            self.stdout.queue(Print(display_line)).unwrap();
        }
        
        // Position cursor at the correct location
        self.stdout.queue(cursor::MoveTo(cursor_x as u16, cursor_y as u16)).unwrap();
        self.stdout.queue(cursor::Show).unwrap();
        self.stdout.flush().unwrap();
       
    }
}