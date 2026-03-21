use crate::buffer;
use crate::viewport;
use crate::terminal;
use std::env;
use std::io::{BufRead, stdout};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};



pub struct Editor {
    pub buffers: Vec<buffer::Buffer>,
    pub current_buffer_index: usize,
    pub viewport: viewport::Viewport,
    pub terminal: terminal::Terminal,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub preferred_x: usize,
}
impl Editor {
    pub fn new() -> Self {
        Editor {
            buffers: Vec::new(),
            current_buffer_index: 0,
            viewport: viewport::Viewport::new(0, 0, 0, 0),
            terminal: terminal::Terminal::new(std::io::stdout()),
            cursor_x: 0,
            cursor_y: 0,
            preferred_x: 0,
        }
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

        enable_raw_mode().unwrap();
        execute!(stdout(), EnterAlternateScreen).unwrap();

        loop {
            self.terminal.draw(
                &self.viewport,
                &self.buffers[self.current_buffer_index],
                self.cursor_x,
                self.cursor_y,
            );

            if event::poll(std::time::Duration::from_millis(50)).unwrap() {
                if let Event::Key(key_event) = event::read().unwrap() {
                    match key_event.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Up => self.move_cursor_up(),
                        KeyCode::Down => self.move_cursor_down(),
                        KeyCode::Left => self.move_cursor_left(),
                        KeyCode::Right => self.move_cursor_right(),
                        KeyCode::Char(c) => {
                            let buffer = &mut self.buffers[self.current_buffer_index];
                            buffer.insert_char(self.cursor_y, self.cursor_x, c);
                            self.move_cursor_right();
                        },
                        KeyCode::Backspace => {
                            if self.cursor_x > 0 {
                                let buffer = &mut self.buffers[self.current_buffer_index];
                                buffer.delete_char(self.cursor_y, self.cursor_x - 1);
                                self.move_cursor_left();
                            } else if self.cursor_y > 0 {
                                let buffer = &mut self.buffers[self.current_buffer_index];
                                let prev_line_len = buffer.data[self.cursor_y - 1].len();
                                buffer.merge_lines(self.cursor_y);
                                self.cursor_y -= 1;
                                self.cursor_x = prev_line_len;
                                self.preferred_x = self.cursor_x;
                            }
                        },
                        KeyCode::Enter => {
                            let buffer = &mut self.buffers[self.current_buffer_index];
                            buffer.split_line(self.cursor_y, self.cursor_x);
                            self.cursor_y += 1;
                            self.cursor_x = 0;
                            self.preferred_x = 0;
                        },
                        _ => {}
                    }
                }
            }
        }

        disable_raw_mode().unwrap();
        execute!(stdout(), LeaveAlternateScreen).unwrap();
    }

    fn move_cursor_up(&mut self) {
        let buffer = &self.buffers[self.current_buffer_index];
        if self.cursor_y > 0 {
            self.cursor_y -= 1;
            let line_len = buffer.data[self.cursor_y].len();
            self.cursor_x = self.preferred_x.min(line_len);
        }
    }

    fn move_cursor_down(&mut self) {
        let buffer = &self.buffers[self.current_buffer_index];
        if self.cursor_y < buffer.data.len() - 1 {
            self.cursor_y += 1;
            let line_len = buffer.data[self.cursor_y].len();
            self.cursor_x = self.preferred_x.min(line_len);
        }
    }

    fn move_cursor_left(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
            self.preferred_x = self.cursor_x;
        }
    }

    fn move_cursor_right(&mut self) {
        let buffer = &self.buffers[self.current_buffer_index];
        if self.cursor_y < buffer.data.len() {
            let line_len = buffer.data[self.cursor_y].len();
            if self.cursor_x < line_len {
                self.cursor_x += 1;
                self.preferred_x = self.cursor_x;
            }
        }
    }

    fn add_buffer(&mut self, buffer: buffer::Buffer) {
        self.buffers.push(buffer);
    }

    fn open_file(&mut self, file_path: &str) {
        let file = std::fs::File::open(file_path).expect("Failed to open file");
        let reader = std::io::BufReader::new(file);
        let mut buffer = buffer::Buffer::new(file_path.to_string());
        buffer.data = reader.lines().collect::<Result<Vec<String>, _>>().unwrap();

        self.add_buffer(buffer);
    }
}