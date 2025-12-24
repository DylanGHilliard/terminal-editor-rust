use std::io::{ Write};
use crossterm::{cursor, terminal::{self, ClearType},
                QueueableCommand, ExecutableCommand,
                style::{self, Print}};  

use crate::{buffer, editor::Editor, viewport};
pub struct Terminal {
    stdout: std::io::Stdout,
    pub size: (u16, u16),
}
impl Terminal {
    pub fn new(stdout: std::io::Stdout) -> Terminal {
        Terminal { stdout, size: terminal::size().unwrap() }
    }

    pub fn draw(&mut self, viewport: &viewport::Viewport, buffer: &buffer::Buffer,) {
         
        self.stdout.queue(cursor::MoveTo(0, 0)).unwrap();
        self.stdout.queue(terminal::Clear(ClearType::All)).unwrap();
        
        

        for (i, line) in buffer.data.iter().enumerate().skip(viewport.offset_y).take(self.size.1 as usize) {

            let  display_line = if line.len() > self.size.0 as usize {
                    &line[viewport.offset_x..self.size.0 as usize]
            } else if line.len() > viewport.offset_x 
                { &line[viewport.offset_x..] }
             else { "" };
            self.stdout.queue(cursor::MoveTo(0, (i - viewport.offset_y) as u16 )).unwrap();
            self.stdout.queue(Print(display_line)).unwrap();
        }
        self.stdout.flush().unwrap();
       
    }
}