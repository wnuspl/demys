use std::fs;
use std::fs::read_dir;
use std::io::Cursor;
use std::path::PathBuf;
use crossterm::event::KeyCode;
use crate::buffer::TextBuffer;
use crate::GridPos;
use crate::style::{StyleItem, };
// holds list of tabs, as well as file system if no tabs are open
// basically just forwards inputs, display requests to correct tab


pub enum WindowRequest {
    Redraw,
    Clear,
    Cursor(Option<GridPos>),
    ReplaceWindow(Box<dyn Window>),
}


pub trait Window {
    fn name(&self) -> String;

    // returns string representation of tab
    fn style(&self, dim: GridPos) -> Vec<StyleItem>;
    fn input(&mut self, key: KeyCode) -> Result<(),String> { Ok(()) }
    fn on_focus(&mut self) {}
    fn leave_focus(&mut self) {}

    fn poll(&mut self) -> Vec<WindowRequest> { Vec::new() }
}

// wraps text to new line past width
// helper function for Tab::display
pub fn wrap(text: &str, width: u16) -> String {
    let width = width as usize;
    let mut out = String::new();
    for line in text.split("\n") {
        // loop until remaining string can be fit
        let mut split = Vec::new();
        let mut remaining: String = line.to_string();
        while remaining.len() > width {
            split.push(remaining[..width].to_string());
            remaining.replace_range(0..width, "");
        }

        split.push(remaining);
        out += &split.join("\n");
    }
    out
}

pub fn pad(text: &str, dim: GridPos) -> String {
    let mut out = String::new();
    let mut count = 0;
    for line in text.split("\n") {
        out += &format!("{:<w$}\n", line, w=dim.col as usize);
        count += 1;
    }


    if count < dim.col {
        for _ in 0..(dim.col-count) {
            out += "\n";
        }
    }

    out
}


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









pub struct FSTab {
    line: u16,
    dir: PathBuf,
    requests: Vec<WindowRequest>,
}


// FILE SYSTEM TAB IMPL
// allows navigation of filesystem to open files
impl FSTab {
    pub fn new(dir: PathBuf) -> FSTab {
        FSTab { line: 0, dir, requests: Vec::new() }
    }
}
impl Window for FSTab {
    fn name(&self) -> String {
        "File Explorer".to_string()
    }
    fn style(&self, dim: GridPos) -> Vec<StyleItem> {
        let mut out = Vec::new();

        if let Ok(dir_iter) = read_dir(&self.dir) {
            for (i, entry) in dir_iter.enumerate() {
                if let Ok(entry) = entry {
                    let name = entry.file_name().to_str().unwrap().to_string();
                    let is_dir = entry.file_type().unwrap().is_dir();

                    let display = if is_dir {
                        format!("> {}/", name)
                    } else {
                        name.to_string()
                    };

                    if self.line == i as u16 {
                        out.push(StyleItem::Color(Some(1)));
                    }

                    out.push(StyleItem::Text(format!("{}", display)));
                    out.push(StyleItem::LineBreak);
                    out.push(StyleItem::Color(None));
                }
            }
        }


        out
    }

    fn poll(&mut self) -> Vec<WindowRequest> {
        std::mem::take(&mut self.requests)
    }


    fn input(&mut self, key: KeyCode) -> Result<(), String> {
        let ret  = match key {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.line > 0 { self.line -= 1; Ok(()) }
                else { Err("".to_string() )}
            },
            KeyCode::Down | KeyCode::Char('j') => {
                // get num of entries
                let dir_len = if let Ok(dir_iter) = read_dir(&self.dir) {
                    dir_iter.count()
                } else { 1 };

                if dir_len != 0 && (self.line as usize) < dir_len-1 { self.line += 1; Ok(()) }
                else { Err("".to_string() )}
            },
            KeyCode::Enter => {
                // change dir to selected
                if let Ok(mut dir_iter) = read_dir(&self.dir) {
                    let selected = dir_iter.nth(self.line as usize).unwrap().unwrap();

                    if ! selected.file_type().unwrap().is_dir() {
                        let opened = Box::new(TextTab::from_file(selected.path()));
                        self.requests.push(WindowRequest::ReplaceWindow(opened));
                    }


                    self.dir = selected.path();
                }

                self.requests.push(WindowRequest::Clear);
                self.line = 0;

                Ok(())
            }
            _ => Err("no match for provided key".to_string())
        };

        self.requests.push(WindowRequest::Redraw);

        ret

    }
}