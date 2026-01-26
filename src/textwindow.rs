use std::fs;
use std::fs::{read_dir, write};
use std::io::Cursor;
use std::path::PathBuf;
use crossterm::event::{KeyCode, KeyModifiers};
use crate::buffer::TextBuffer;
use crate::GridPos;
use crate::style::{StyleItem, };
use crate::window::{Window, WindowRequest, pad, ScrollableData, Scrollable};

pub struct TextWindow {
    tb: TextBuffer,
    requests: Vec<WindowRequest>,
    name: String,
    scrollable_data: ScrollableData,
}



// TEXT TAB IMPL
// Holds text buffers
impl TextWindow {
    pub fn new(tb: TextBuffer) -> TextWindow {
        TextWindow { tb, requests: Vec::new(), name: "".into(), scrollable_data: ScrollableData::default(), }
    }
    pub fn from_file(path: PathBuf) -> TextWindow {
        let name = path.file_name().unwrap().to_string_lossy().into();
        TextWindow { tb: TextBuffer::from(path), requests: Vec::new(), name, scrollable_data: ScrollableData::default() }
    }
}


impl Window for TextWindow {
    fn name(&self) -> String {
        let saved_symbol = if self.tb.saved { "" }
            else { "*" };
        format!("{}{}",saved_symbol,self.name)
    }
    fn style(&self, dim: GridPos) -> Vec<StyleItem> {
        let raw = format!("{}", self.tb);
        let mut out = Vec::new();

        for line in raw.split("\n").skip(self.scrollable_data.scroll_offset as usize) {
            out.push(StyleItem::Text(line.to_string()));
            out.push(StyleItem::LineBreak);
        }

        // pad
        pad(&mut out, dim)
    }
    fn input(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        match key {
            KeyCode::Backspace => { self.tb.delete(1); }
            KeyCode::Enter => {
                self.tb.insert("\n");
                self.scrollable_data.total_lines = self.tb.len() as u16;
               // self.scroll_to(self.tb.cursor.0 as u16);
            }
            KeyCode::F(12) => {
                self.tb.save();
            }

            KeyCode::Up => {
                self.tb.cursor_move_by(Some(-1), None);
                //self.scroll_to(self.tb.cursor.0 as u16);
            }
            KeyCode::Down => {
                self.tb.cursor_move_by(Some(1), None);
                //self.scroll_to(self.tb.cursor.0 as u16);
            }
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
    fn on_resize(&mut self, dim: GridPos) {
        self.scrollable_data.screen_rows = dim.row;
        self.scrollable_data.scroll_margin = 2;
    }
    fn leave_focus(&mut self) {
        self.requests.push(WindowRequest::Cursor(None));
    }
    fn requests(&mut self) -> &mut Vec<WindowRequest> {
        &mut self.requests
    }
}


impl Scrollable for TextWindow {
    fn get_data_mut(&mut self) -> &mut ScrollableData {
        &mut self.scrollable_data
    }
}

