
pub struct Buffer {
    data : Vec<String>,
    file_path : String,

}

impl Buffer {

    pub fn new(file: String) -> Buffer {
        Buffer { data: Vec::new() , file_path: file }
    }

    pub fn write(&mut self, line: &str) {
        self.data.push(line.to_string());
    }

    pub fn read(&self) -> Vec<String> {
        self.data.clone()
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
}