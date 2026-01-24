use std::fs;
use std::fs::read_dir;
use std::io::Cursor;
use std::path::PathBuf;
use crossterm::event::KeyCode;
use crate::buffer::TextBuffer;
use crate::GridPos;
use crate::style::{StyleItem, };
use crate::window::{Window, WindowRequest, pad};

pub struct TextTab {
    tb: TextBuffer,
    name: String,
    requests: Vec<WindowRequest>
}



// TEXT TAB IMPL
// Holds text buffers
impl TextTab {
    pub fn new(tb: TextBuffer, name: String) -> TextTab {
        TextTab { tb, name, requests: Vec::new() }
    }
    pub fn from_file(path: PathBuf) -> TextTab {
        TextTab { tb: TextBuffer::from(path), name: String::new(), requests: Vec::new() }
    }
}

impl Window for TextTab {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn style(&self, dim: GridPos) -> Vec<StyleItem> {
        let raw = pad(&format!("{}",self.tb), dim);
        let mut out = Vec::new();

        for line in raw.split("\n") {
            out.push(StyleItem::Text(line.to_string()));
            //out.push(StyleItem::Text(format!("{:<w$}", line, w=dim.col as usize)));
            out.push(StyleItem::LineBreak);
        }

        out
    }
    fn input(&mut self, key: KeyCode) -> Result<(), String> {
        let ret = match key {
            KeyCode::Backspace => self.tb.delete(1),
            KeyCode::Enter => self.tb.insert("\n"),
            KeyCode::F(12) => {self.tb.save(); Ok(())},
            
            KeyCode::Up => self.tb.cursor_move_by(Some(-1), None),
            KeyCode::Down => self.tb.cursor_move_by(Some(1), None),
            KeyCode::Left => self.tb.cursor_move_by(None, Some(-1)),
            KeyCode::Right => self.tb.cursor_move_by(None, Some(1)),


            KeyCode::Char(ch) => self.tb.insert(&ch.to_string()),
            _ => Err("no match for provided key".to_string())
        };

        self.requests.push(WindowRequest::Cursor(Some((self.tb.cursor.0 as u16, self.tb.cursor.1 as u16).into())));
        self.requests.push(WindowRequest::Redraw);

        ret
    }
    fn on_focus(&mut self) {
        self.requests.push(WindowRequest::Cursor(Some(
            (self.tb.cursor.0 as u16, self.tb.cursor.1 as u16).into()
        )));
    }
    fn leave_focus(&mut self) {
        self.requests.push(WindowRequest::Cursor(None));
    }
    fn poll(&mut self) -> Vec<WindowRequest> {
        std::mem::take(&mut self.requests)
    }
}



