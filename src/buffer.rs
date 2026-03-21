
pub struct Buffer {
    pub data: Vec<String>,
    pub file_path: String,
}

impl Buffer {
    pub fn new(file: String) -> Buffer {
        Buffer {
            data: Vec::new(),
            file_path: file,
        }
    }

    pub fn insert_char(&mut self, line_num: usize, char_index: usize, c: char) {
        if let Some(line) = self.data.get_mut(line_num) {
            line.insert(char_index, c);
        }
    }

    pub fn delete_char(&mut self, line_num: usize, char_index: usize) {
        if let Some(line) = self.data.get_mut(line_num) {
            line.remove(char_index);
        }
    }

    pub fn split_line(&mut self, line_num: usize, char_index: usize) {
        if line_num < self.data.len() {
            let line = &mut self.data[line_num];
            let remainder = line.split_off(char_index);
            self.data.insert(line_num + 1, remainder);
        }
    }

    pub fn merge_lines(&mut self, line_num: usize) {
        if line_num > 0 && line_num < self.data.len() {
            let current_line = self.data.remove(line_num);
            if let Some(prev_line) = self.data.get_mut(line_num - 1) {
                prev_line.push_str(&current_line);
            }
        }
    }
}