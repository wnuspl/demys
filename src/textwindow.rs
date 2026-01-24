use std::fs;
use std::fs::{read_dir, write};
use std::io::Cursor;
use std::path::PathBuf;
use crossterm::event::KeyCode;
use crate::buffer::TextBuffer;
use crate::GridPos;
use crate::style::{StyleItem, };
use crate::window::{Window, WindowRequest, pad};

pub struct TextWindow {
    tb: TextBuffer,
    requests: Vec<WindowRequest>,
    name: String
}



// TEXT TAB IMPL
// Holds text buffers
impl TextWindow {
    pub fn new(tb: TextBuffer) -> TextWindow {
        TextWindow { tb, requests: Vec::new(), name: "".into()}
    }
    pub fn from_file(path: PathBuf) -> TextWindow {
        let name = path.file_name().unwrap().to_string_lossy().into();
        TextWindow { tb: TextBuffer::from(path), requests: Vec::new(), name }
    }
}


impl Window for TextWindow {
    fn name(&self) -> String {
        let saved_symbol = if self.tb.saved { "" }
            else { "*" };
        format!("{}{}",saved_symbol,self.name)
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
    fn input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Backspace => { self.tb.delete(1); }
            KeyCode::Enter => { self.tb.insert("\n"); }
            KeyCode::F(12) => {
                self.tb.save();
            }

            KeyCode::Up => { self.tb.cursor_move_by(Some(-1), None); }
            KeyCode::Down => { self.tb.cursor_move_by(Some(1), None); }
            KeyCode::Left => { self.tb.cursor_move_by(None, Some(-1)); }
            KeyCode::Right => { self.tb.cursor_move_by(None, Some(1)); }

            KeyCode::Char(ch) => { self.tb.insert(&ch.to_string()); }
            _ => ()
        };

        self.requests.push(WindowRequest::Redraw);
        self.requests.push(WindowRequest::Cursor(Some(
            (self.tb.cursor.0 as u16, self.tb.cursor.1 as u16).into()
        )));
    }
    fn on_focus(&mut self) {
        self.requests.push(WindowRequest::Cursor(Some(
            (self.tb.cursor.0 as u16, self.tb.cursor.1 as u16).into()
        )));
    }
    fn leave_focus(&mut self) {
        self.requests.push(WindowRequest::Cursor(None));
    }
    fn requests(&mut self) -> &mut Vec<WindowRequest> {
        &mut self.requests
    }
}



