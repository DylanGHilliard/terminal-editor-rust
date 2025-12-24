use crate::buffer;
use crate::viewport;
use crate::terminal;
use std::env;
use std::io::{Read, Write, BufRead, BufReader};



pub struct Editor {
    pub buffers: Vec<buffer::Buffer>,
    pub current_buffer_index: usize,
    pub viewport: viewport::Viewport,
    pub terminal: terminal::Terminal,
}
impl Editor {
    pub fn new() -> Self {
        Editor { buffers: Vec::new(),
            current_buffer_index: 0, 
            viewport: viewport::Viewport::new(0, 0, 0, 0),
            terminal: terminal::Terminal::new(std::io::stdout()), }
    }


    pub fn main_loop(&mut self) {
        let args: Vec<String> = env::args().collect();
       
        if args.len() > 1 {
            self.open_file(&args[1]);
        }
        else{
            self.open_file("textfiles/test2.txt");
        }

        self.viewport.set_size(self.terminal.size.0 as usize, self.terminal.size.1 as usize);


        loop {
            self.terminal.draw(&self.viewport, &self.buffers[self.current_buffer_index]);   
            
        }
    }

    pub fn add_buffer(&mut self, buffer: buffer::Buffer) {
        self.buffers.push(buffer);
    }
    
    pub fn open_file(&mut self, file_path: &str) {
        let mut file = std::fs::File::open(file_path).expect("Failed to open file");
        let reader = std::io::BufReader::new(file);
        let mut buffer = buffer::Buffer::new(file_path.to_string());
        buffer.data = reader.lines().collect::<Result<Vec<String>, _>>().unwrap();

        self.add_buffer(buffer);
        
    }
}