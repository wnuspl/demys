use std::fs::read_dir;
use std::path::PathBuf;
use crossterm::event::KeyCode;
use crate::buffer::TextBuffer;
use crate::GridPos;
use crate::style::{StyleItem, };
// holds list of tabs, as well as file system if no tabs are open
// basically just forwards inputs, display requests to correct tab


pub enum WindowRequest {
    Redraw,
    ReplaceWindow(Box<dyn Window>),
}


pub trait Window {
    fn name(&self) -> String;

    // returns string representation of tab
    fn style(&self, dim: GridPos) -> Vec<StyleItem>;
    fn input(&mut self, key: KeyCode) -> Result<(),String> { Ok(()) }

    // none if cursor should be hidden (0 is row, 1 is col)
    fn cursor_location(&self) -> Option<GridPos> { None }

    fn poll(&self) -> Option<Vec<WindowRequest>> { None }

    // wraps text to new line past width
    // helper function for Tab::display
    fn wrap(&self, text: &str, width: u16) -> String {
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
}


pub struct FSTab {
    line: u16,
    dir: PathBuf
}

pub struct TextTab {
    tb: TextBuffer,
    name: String
}



// TEXT TAB IMPL
// Holds text buffers
impl TextTab {
    pub fn new(tb: TextBuffer, name: String) -> TextTab {
        TextTab { tb, name }
    }
}

impl Window for TextTab {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn style(&self, dim: GridPos) -> Vec<StyleItem> {
        let raw = format!("{}",self.tb);
        let mut out = Vec::new();

        // add line numbers
        let mut line_number = 0;
        for line in raw.split("\n") {

            line_number += 1;
            out.push(StyleItem::Text(format!("{} | {}", line_number, self.wrap(line, dim.col))));
            out.push(StyleItem::LineBreak);
        }

        out
    }
    fn input(&mut self, key: KeyCode) -> Result<(), String> {
        match key {
            KeyCode::Backspace => self.tb.delete(1),
            KeyCode::Enter => self.tb.insert("\n"),
            KeyCode::Char(ch) => self.tb.insert(&ch.to_string()),
            _ => Err("no match for provided key".to_string())
        }
    }
    fn cursor_location(&self) -> Option<GridPos> {
        Some((self.tb.cursor.0 as u16, self.tb.cursor.1 as u16+4).into())
    }
}





pub struct CharTab(pub char);

impl Window for CharTab {
    fn name(&self) -> String { "char".to_string() }

    fn style(&self, dim: GridPos) -> Vec<StyleItem> {
        let mut out = Vec::new();
        out.push(StyleItem::Color(0));
        for i in 0..dim.row {
            for j in 0..dim.col {
                out.push(StyleItem::Text(self.0.to_string()));
            }
            if i+1 != dim.row { out.push(StyleItem::LineBreak); }
        }
        out.push(StyleItem::Color(1));
        out
    }
}










// FILE SYSTEM TAB IMPL
// allows navigation of filesystem to open files
impl FSTab {
    pub fn new(dir: PathBuf) -> FSTab {
        FSTab { line: 0, dir }
    }
}
impl Window for FSTab {
    fn name(&self) -> String {
        "File Explorer".to_string()
    }
    fn style(&self, dim: GridPos) -> Vec<StyleItem> {
        let mut out = Vec::new();

        if let Ok(dir_iter) = read_dir(&self.dir) {
            for entry in dir_iter {
                if let Ok(entry) = entry {
                    let name = entry.file_name().to_str().unwrap().to_string();
                    let is_dir = entry.file_type().unwrap().is_dir();

                    let display = if is_dir {
                        format!("> {}/", name)
                    } else {
                        name.to_string()
                    };

                    out.push(StyleItem::Text(format!("{}", display)));
                    out.push(StyleItem::LineBreak);
                }
            }
        }


        out
    }


    fn input(&mut self, key: KeyCode) -> Result<(), String> {
        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.line > 0 { self.line -= 1; Ok(()) }
                else { Err("".to_string() )}
            },
            KeyCode::Down | KeyCode::Char('j') => {
                // get num of entries
                let dir_len = if let Ok(mut dir_iter) = read_dir(&self.dir) {
                    dir_iter.count()
                } else { 1 };

                if (self.line as usize) < dir_len-1 { self.line += 1; Ok(()) }
                else { Err("".to_string() )}
            },
            KeyCode::Enter => {
                // change dir to selected
                if let Ok(mut dir_iter) = read_dir(&self.dir) {
                    let selected = dir_iter.nth(self.line as usize).unwrap().unwrap().path();
                    self.dir = selected;
                }

                Ok(())
            }
            _ => Err("no match for provided key".to_string())
        }

    }

    fn cursor_location(&self) -> Option<GridPos> {
        Some((self.line, 0).into())
    }

}