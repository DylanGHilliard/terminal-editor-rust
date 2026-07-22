use std::io;

/// Clipboard access is abstract so an OS or terminal clipboard can replace the
/// in-memory implementation without changing editing code.
pub trait Clipboard {
    fn read(&mut self) -> io::Result<String>;
    fn write(&mut self, text: &str) -> io::Result<()>;
}

#[derive(Debug, Default)]
pub struct MemoryClipboard {
    contents: String,
}

impl Clipboard for MemoryClipboard {
    fn read(&mut self) -> io::Result<String> {
        Ok(self.contents.clone())
    }

    fn write(&mut self, text: &str) -> io::Result<()> {
        text.clone_into(&mut self.contents);
        Ok(())
    }
}
