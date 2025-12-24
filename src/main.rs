


mod buffer;
mod editor;
mod viewport;
mod terminal;



struct Editor {
    buffer: Vec<String>,
    cursor_x: usize,
    cursor_y: usize,
    offset_x: usize,
    offset_y: usize,
    terminal_width: usize,
    terminal_height: usize,
    filename : Option<String>,
}

//impl Editor {
//    fn new() -> Self {
//        Self {
//            buffer: vec![String::new()],
//            cursor_x: 0,
//            cursor_y: 0,
//            offset_x: 0,
//            offset_y: 0,
//            terminal_height: terminal::size().unwrap().1 as usize,
//            terminal_width: terminal::size().unwrap().0 as usize,
//            filename: None,
//        }
//    }
//    fn open(&mut self, filename: &str) -> io::Result<()> {
//        let file = File::open(filename)?;
//        let reader = BufReader::new(file);
//        self.buffer = reader.lines().collect::<Result<_, _>>()?;
//        self.filename = Some(filename.to_string());
//        Ok(())
//    }
//
//    fn save(&self) -> io::Result<()> {
//        if let Some(filename) = &self.filename {
//            let mut file = File::create(filename)?;
//            for line in &self.buffer {
//                writeln!(file, "{}", line)?;
//            }
//        }
//        Ok(())
//    }
//
//    fn delete_char(&mut self) {
//        if self.cursor_y >= self.buffer.len()    { return; }
//
//        
//        if self.cursor_x + self.offset_x > 0 {
//            let line = &mut self.buffer[self.cursor_y + self.offset_y];
//            line.remove(self.cursor_x - 1 + self.offset_x);
//            self.cursor_x -= 1;
//        } else if self.cursor_y + self.offset_y > 0 {
//            let prev_line_len: usize =  self.buffer[self.cursor_y + self.offset_y - 1].len();
//            let current_line = self.buffer[self.cursor_y + self.offset_y].clone();
//            self.buffer.remove(self.cursor_y + self.offset_y);
//            if self.cursor_y > 0 {
//                self.cursor_y -= 1;
//            } else {
//                self.offset_y -= 1;
//            }
//            self.cursor_x = prev_line_len;
//            self.buffer[self.cursor_y + self.offset_y].push_str(&current_line);
//        }
//    }
//
//    fn insert_newline(&mut self) {
//        if self.cursor_y + self.offset_y >= self.buffer.len() { return; }
//
//        let line = &mut self.buffer[self.cursor_y + self.offset_y];
//        let new_line = line.split_off(self.cursor_x + self.offset_x);
//        self.buffer.insert(self.cursor_y + self.offset_y + 1, new_line);
//
//        if self.cursor_y < self.terminal_height -1 {
//            self.cursor_y += 1;
//        }
//        else{
//            self.offset_y += 1;
//        }
//
//        self.cursor_x = 0;
//    }
//
//
//
//    fn insert_char(&mut self, c: char) {
//        if self.cursor_y + self.offset_y >= self.buffer.len() {
//            self.buffer.push(String::new());
//        }
//        let line = &mut self.buffer[self.cursor_y + self.offset_y];
//        line.insert(self.cursor_x + self.offset_x, c);
//        if self.cursor_x < self.terminal_width - 1 {
//            self.cursor_x += 1;
//        } else {
//            self.offset_x += 1;
//        }
//        
//    }
//
//
//     fn move_left(&mut self) {
//        if self.cursor_x > 0 { self.cursor_x -= 1; }
//        else if self.offset_x > 0 {
//            self.offset_x -= 1;
//        }
//        else if self.cursor_y > 0 {
//        self.cursor_y -= 1;
//        self.offset_x = self.buffer[self.cursor_y].len().saturating_sub(self.terminal_width);
//        self.cursor_x = self.terminal_width.min(self.buffer[self.cursor_y].len());
//        }
//    }
//
//    fn move_right(&mut self) {
//
//        if self.cursor_x + self.offset_x < self.buffer[self.cursor_y].len() {
//            if self.cursor_x < self.terminal_width - 1 {
//                self.cursor_x += 1;
//            }
//            else  {
//                self.offset_x += 1;
//            }
//        }
//
//        else if self.cursor_y + 1 < self.buffer.len() {
//            self.cursor_y += 1;
//            self.cursor_x = 0;
//            self.offset_x = 0;
//        }
//    }
//    fn move_up(&mut self) {
//        if self.cursor_y > 0 {
//        
//            self.cursor_y -= 1;
//            self.cursor_x = self.cursor_x.min(self.buffer[self.cursor_y + self.offset_y].len());
//        }
//        else if self.offset_y > 0 {
//            self.offset_y -= 1;
//            self.cursor_x = self.cursor_x.min(self.buffer[self.cursor_y + self.offset_y].len());
//        }
//    }
//
//    fn move_down(&mut self) {
//        if self.cursor_y + 1 < self.terminal_height && self.cursor_y + 1 < self.buffer.len() {
//            self.cursor_y += 1;
//            self.cursor_x = self.cursor_x.min(self.buffer[self.cursor_y+self.offset_y].len());
//        }
//        else  if self.cursor_y + self.offset_y + 1 < self.buffer.len() {
//            self.offset_y += 1;
//            self.cursor_x = self.cursor_x.min(self.buffer[self.cursor_y + self.offset_y].len());
//        }
//    }
//
//     fn draw<W: Write>(&self, stdout: &mut W) -> io::Result<()> {
//        // Clear the terminal before drawing to avoid appending/staggering
//        stdout.queue(terminal::Clear(terminal::ClearType::All))?;
//
//        for (i, line) in self.buffer.iter().enumerate().skip(self.offset_y).take(self.terminal_height) {
//
//             let display_line =  if line.len() > self.terminal_width {
//                &line[self.offset_x..self.terminal_width + self.offset_x - 1]
//            }
//            else if line.len() > self.offset_x
//                {&line[self.offset_x..]}
//            else { "" };
//
//            stdout.queue(cursor::MoveTo(0, (i - self.offset_y) as u16))?;
//            stdout.queue(Print(display_line))?;
//        }
//
//
//        stdout.flush()?;
//        Ok(())
//    }
//    
//
//}   


fn main()  {

    let mut editor = editor::Editor::new();
    editor.main_loop();


    
//    let mut stdout = stdout();
//    enable_raw_mode()?;
//    execute!(stdout, terminal::EnterAlternateScreen, event::EnableMouseCapture, cursor::Show, cursor::MoveTo(0,0))?;
//    
//
//    loop {
//       
//        editor.draw(&mut stdout)?;
//        
//        execute!(stdout, cursor::MoveTo(editor.cursor_x as u16, editor.cursor_y as u16))?;
//
//        match read()? {
//            Event::Key(KeyEvent {code, modifiers, kind: _, state: _}) => match (code, modifiers) {
//                (KeyCode::Char('q'), KeyModifiers::CONTROL) => {break},
//                (KeyCode::Left, _) => editor.move_left(),
//                (KeyCode::Right, _) => editor.move_right(),
//                (KeyCode::Up, _) => editor.move_up(),
//                (KeyCode::Down, _) => editor.move_down(),
//                (KeyCode::Backspace, _) => editor.delete_char(),
//                (KeyCode::Char('s'), KeyModifiers::CONTROL) => editor.save()?,
//                (KeyCode::Char(c), _)  => editor.insert_char(c),
//                (KeyCode::Enter, _) => editor.insert_newline(),
//                _  => {}
//            },
//                Event::Mouse(mouse_event)=>  {
//                match mouse_event.kind {
//                    event::MouseEventKind::Down(MouseButton::Left) => {
//                        editor.cursor_x = mouse_event.column as usize;
//                        editor.cursor_y = mouse_event.row as usize;
//                        editor.cursor_y = editor.cursor_y.min(editor.buffer.len() - 1);
//                        editor.cursor_x = editor.cursor_x.min(editor.buffer[editor.cursor_y].len());
//                        
//                    },
//                    event::MouseEventKind::ScrollDown => editor.move_down(),
//                    event::MouseEventKind::ScrollUp => editor.move_up(),
//                    _ => {}
//                }
//            },
//             _ => {}
//        }
//    }
//    disable_raw_mode()?;
//    execute!(stdout, terminal::LeaveAlternateScreen, event::DisableMouseCapture)?;
//    println!("Exiting editor...");
//    Ok(())
}