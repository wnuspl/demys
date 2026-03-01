use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use crate::textedit::buffer::TextBuffer;
use crate::textedit::operation::TextBufferOperation;

pub struct TextFile {
    path: PathBuf,
    buffer: TextBuffer,
    saved: bool
}

impl From<PathBuf> for TextFile {
    fn from(path: PathBuf) -> Self {
        Self {
            path: path.clone(),
            buffer: path.into(),
            saved: true
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
