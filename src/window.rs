use std::fs;
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

    fn poll(&mut self) -> Vec<WindowRequest> { Vec::new() }

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
        let file = fs::File::open(path).unwrap();

        TextTab { tb: TextBuffer::from(file), name: String::new(), requests: Vec::new() }
    }
}

impl Window for TextTab {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn style(&self, dim: GridPos) -> Vec<StyleItem> {
        let raw = format!("{}",self.tb);
        let mut out = Vec::new();

        for line in raw.split("\n") {
            out.push(StyleItem::Text(format!("{:<w$}", line, w=dim.col as usize)));
            out.push(StyleItem::LineBreak);
        }

        out
    }
    fn input(&mut self, key: KeyCode) -> Result<(), String> {
        self.requests.push(WindowRequest::Redraw);
        match key {
            KeyCode::Backspace => self.tb.delete(1),
            KeyCode::Enter => self.tb.insert("\n"),
            KeyCode::Char(ch) => self.tb.insert(&ch.to_string()),
            _ => Err("no match for provided key".to_string())
        }
    }
    fn cursor_location(&self) -> Option<GridPos> {
        Some((self.tb.cursor.0 as u16, self.tb.cursor.1 as u16).into())
    }
    fn poll(&mut self) -> Vec<WindowRequest> {
        std::mem::take(&mut self.requests)
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






pub struct FSTab {
    line: u16,
    dir: PathBuf,
    selected: Vec<WindowRequest>,
}


// FILE SYSTEM TAB IMPL
// allows navigation of filesystem to open files
impl FSTab {
    pub fn new(dir: PathBuf) -> FSTab {
        FSTab { line: 0, dir, selected: Vec::new() }
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

    fn poll(&mut self) -> Vec<WindowRequest> {
        std::mem::take(&mut self.selected)
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
                    let selected = dir_iter.nth(self.line as usize).unwrap().unwrap();

                    if ! selected.file_type().unwrap().is_dir() {
                        let opened = Box::new(TextTab::from_file(selected.path()));
                        self.selected.push(WindowRequest::ReplaceWindow(opened));
                    }


                    self.dir = selected.path();
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