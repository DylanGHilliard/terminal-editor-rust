use crate::buffer;
use std::io::{Read, Write};


pub struct Editor {
    buffers: Vec<buffer::Buffer>,
}
impl Editor {
    pub fn new() -> Self {
        Editor { buffers: Vec::new() }
    }

    pub fn add_buffer(&mut self, buffer: buffer::Buffer) {
        self.buffers.push(buffer);
    }
    
    pub fn open_file(&mut self, file_path: &str) {

    }
}