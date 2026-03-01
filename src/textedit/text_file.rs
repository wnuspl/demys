use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use crate::textedit::buffer::TextBuffer;
use crate::textedit::operation::{CursorLeft, InsertString, TextBufferOperation};

pub struct TextFile {
    path: PathBuf,
    buffer: TextBuffer,
    saved: bool
}

impl From<PathBuf> for TextFile {
    fn from(path: PathBuf) -> Self {
        let mut buffer = TextBuffer::new();
        let mut saved = false;
        if fs::exists(&path).unwrap() {
            let content = fs::read_to_string(&path).unwrap();
            buffer.apply(Box::new(InsertString::new(content)));
            let length = buffer.get_length();
            buffer.apply(Box::new(CursorLeft(length)));
            saved = true;
        }
        Self {
            path,
            buffer,
            saved
        }
    }
}

impl TextFile {
    pub fn apply(&mut self, operation: Box<dyn TextBufferOperation>) {
        let modifies = operation.modifies();
        let result = self.buffer.apply(operation);

        if result.is_ok() && modifies {
            self.saved = false;
        }
    }
    pub fn undo(&mut self) {
        let result = self.buffer.undo();
        if let Ok(operation) = result {
            if operation.modifies() {
                self.saved = false;
            }
        }
    }
    pub fn buffer(&self) -> &TextBuffer {
        &self.buffer
    }
    pub fn save(&mut self) {
        if self.saved {
            return;
        }
        if let Ok(mut file) = File::create(&self.path) {
            let text = self.buffer.string();
            let data = text.as_bytes();

            file.write_all(data);

            self.saved = true;
        }
    }

    pub fn saved(&self) -> bool {
        self.saved
    }
}
