use std::io::{self, Write, stdout, BufRead, BufReader};
use std::fs::File;
use crossterm::{
        ExecutableCommand, QueueableCommand, cursor, event, execute, terminal,
        event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
        terminal::{disable_raw_mode, enable_raw_mode},
    };

use crossterm::style::{Attribute, Print, SetAttribute};




struct Editor {
    buffer: Vec<String>,
    cursor_x: usize,
    cursor_y: usize,
    offset_x: usize,
    offset_y: usize,
    filename : Option<String>,
}

impl Editor {
    fn new() -> Self {
        Self {
            buffer: vec![String::new()],
            cursor_x: 0,
            cursor_y: 0,
            offset_x: 0,
            offset_y: 0,
            filename: None,
        }
    }
    fn open(&mut self, filename: &str) -> io::Result<()> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        self.buffer = reader.lines().collect::<Result<_, _>>()?;
        self.filename = Some(filename.to_string());
        Ok(())
    }

    fn save(&self) -> io::Result<()> {
        if let Some(filename) = &self.filename {
            let mut file = File::create(filename)?;
            for line in &self.buffer {
                writeln!(file, "{}", line)?;
            }
        }
        Ok(())
    }

    fn delete_char(&mut self) {
        if self.cursor_y >= self.buffer.len()    { return; }

        
        if self.cursor_x > 0 {
            let line = &mut self.buffer[self.cursor_y];
            line.remove(self.cursor_x - 1);
            self.cursor_x -= 1;
        } else if self.cursor_y > 0 {
            let prev_line_len: usize =  self.buffer[self.cursor_y - 1].len();
            let current_line = self.buffer[self.cursor_y].clone();
            self.buffer.remove(self.cursor_y);
            self.cursor_y -= 1;
            self.cursor_x = prev_line_len;
            self.buffer[self.cursor_y].push_str(&current_line);
        }
    }

    fn insert_newline(&mut self) {
        if self.cursor_y >= self.buffer.len() { return; }
        let line = &mut self.buffer[self.cursor_y];
        let new_line = line.split_off(self.cursor_x);
        self.buffer.insert(self.cursor_y + 1, new_line);
        self.cursor_y += 1;
        self.cursor_x = 0;
    }



    fn insert_char(&mut self, c: char) {
        if self.cursor_y >= self.buffer.len() {
            self.buffer.push(String::new());
        }
        let line = &mut self.buffer[self.cursor_y];
        line.insert(self.cursor_x, c);
        self.cursor_x += 1;
    }


     fn move_left(&mut self) {
        if self.cursor_x > 0 { self.cursor_x -= 1; }
        else if self.cursor_y > 0 {
        self.cursor_y -= 1;
        self.cursor_x = self.buffer[self.cursor_y].len();
        }
    }

    fn move_right(&mut self) {
        if self.cursor_y >= self.buffer.len() { return; }

        if self.cursor_x < self.buffer[self.cursor_y].len() { self.cursor_x += 1; }

        else if self.cursor_y + 1 < self.buffer.len() {
            self.cursor_y += 1;
            self.cursor_x = 0;
        }
    }
    fn move_up(&mut self) {
        if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.cursor_x = self.cursor_x.min(self.buffer[self.cursor_y].len());
        }
    }

    fn move_down(&mut self) {
        if self.cursor_y + 1 < self.buffer.len() {
            self.cursor_y += 1;
            self.cursor_x = self.cursor_x.min(self.buffer[self.cursor_y].len());
        }
    }

     fn draw<W: Write>(&self, stdout: &mut W) -> io::Result<()> {
        // Clear the terminal before drawing to avoid appending/staggering
        stdout.queue(terminal::Clear(terminal::ClearType::All))?;

        // Draw each line at column 0 explicitly so wrapped/long lines don't shift subsequent lines
        for (i, line) in self.buffer.iter().enumerate() {
            stdout.queue(cursor::MoveTo(0, i as u16))?;
            stdout.queue(Print(line))?;
        }
        
        stdout.flush()?;
        Ok(())
    }
    

}   


fn main() -> io::Result<()> {
    let mut editor = Editor::new();
    editor.open("textfiles/test.txt")?; 
    
    let mut stdout = stdout();
    enable_raw_mode()?;
    execute!(stdout, terminal::EnterAlternateScreen, event::EnableMouseCapture, cursor::Show, cursor::MoveTo(0,0))?;
    

    loop {
       
        editor.draw(&mut stdout)?;
        
        execute!(stdout, cursor::MoveTo(editor.cursor_x as u16, editor.cursor_y as u16))?;

        match read()? {
            Event::Key(KeyEvent {code, modifiers, kind: _, state: _}) => match (code, modifiers) {
                (KeyCode::Char('q'), KeyModifiers::CONTROL) => {editor.save();break},
                (KeyCode::Left, _) => editor.move_left(),
                (KeyCode::Right, _) => editor.move_right(),
                (KeyCode::Up, _) => editor.move_up(),
                (KeyCode::Down, _) => editor.move_down(),
                (KeyCode::Char(c), _) if c.is_alphabetic() => editor.insert_char(c),
                (KeyCode::Backspace, _) => editor.delete_char(),
                (KeyCode::Char('s'), KeyModifiers::CONTROL) => editor.save()?,
                (KeyCode::Enter, _) => editor.insert_newline(),
                    
                _  => {}
            },
             _ => {}
        }
    }
    disable_raw_mode()?;
    execute!(stdout, terminal::LeaveAlternateScreen, event::DisableMouseCapture)?;
    println!("Exiting editor...");
    Ok(())
}