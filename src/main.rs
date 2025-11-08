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

     fn draw<W: Write>(&self, stdout: &mut W) -> io::Result<()> {
        for (i, line) in self.buffer.iter().enumerate() {
            stdout.queue(Print(format!("{}\n", line)))?;
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
    execute!(stdout, terminal::EnterAlternateScreen, event::EnableMouseCapture, cursor::Show)?;


    loop {
       
        editor.draw(&mut stdout)?;
        let cursor_x = (editor.cursor_x - editor.offset_x) as u16;
        let cursor_y = (editor.cursor_y - editor.offset_y) as u16;
        //execute!(stdout, cursor::MoveTo(cursor_x, cursor_y))?;

        match read()? {
            Event::Key(KeyEvent {code, modifiers, kind: _, state: _}) => match (code, modifiers) {
                (KeyCode::Char('q'), KeyModifiers::CONTROL) => break,
                (KeyCode::Left, _) => editor.move_left(),
                (KeyCode::Right, _) => {execute!(stdout, cursor::MoveRight(1))?;},
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