use std::fs::read_dir;
use std::path::PathBuf;
use console::Key;
use crate::buffer::TextBuffer;


// holds list of tabs, as well as file system if no tabs are open
// basically just forwards inputs, display requests to correct tab
pub struct Window {
    pub tabs: Vec<TextTab>,
    pub current_tab: Option<usize>,
    fs_tab: FSTab,
    wrap_text: bool,
}

struct FSTab {
    line: usize,
    dir: PathBuf
}

pub struct TextTab {
    tb: TextBuffer,
    name: String
}

trait Tab {
    fn name(&self) -> String;

    // returns string representation of tab
    fn display(&self, width: usize, height: usize) -> String;
    fn input(&mut self, key: Key) -> Result<(),String> { Ok(()) }

    // none if cursor should be hidden (0 is row, 1 is col)
    fn cursor_location(&self) -> Option<(usize, usize)> { None }

    // wraps text to new line past width
    // helper function for Tab::display
    fn wrap(text: &str, width: usize) -> String {
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






impl Window {
    pub fn new() -> Window {
        Window {
            tabs: vec![TextTab::new(TextBuffer::new(), "main".to_string())],
           // current_tab: Some(0),
            current_tab: None,
            fs_tab: FSTab::new("/".into()),
            wrap_text: false,
        }
    }

    pub fn input(&mut self, key: Key) -> Result<(),String> {
        if let Some(tab) = self.current_tab {
            self.tabs[tab].input(key)
        } else {
            self.fs_tab.input(key)
        }
    }


    // calls appropriate tab to get text,
    // wraps or cuts to fit width/height
    pub fn display(&self, width: usize, height: usize) -> String {
        if let Some(tab) = self.current_tab {
            self.tabs[tab].display(width, height)
        } else {
            self.fs_tab.display(width, height)
        }
    }

    pub fn cursor_location(&self) -> Option<(usize, usize)> {
        if let Some(tab) = self.current_tab {
            self.tabs[tab].cursor_location()
        } else {
            self.fs_tab.cursor_location()
        }
    }
}








// TEXT TAB IMPL
// Holds text buffers
impl TextTab {
    pub fn new(tb: TextBuffer, name: String) -> TextTab {
        TextTab { tb, name }
    }
}

impl Tab for TextTab {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn display(&self, width: usize, height: usize) -> String {
        let raw = format!("{}",self.tb);
        let mut out = String::new();

        // add line numbers
        let mut line_number = 0;
        for line in raw.split("\n") {

            line_number += 1;
            out += &format!("{} | {}\n", line_number, Self::wrap(line, width));
        }

        out
    }
    fn input(&mut self, key: Key) -> Result<(), String> {
        match key {
            Key::Backspace => self.tb.delete(1),
            Key::Enter => self.tb.insert("\n"),
            Key::Char(ch) => self.tb.insert(&ch.to_string()),
            _ => Err("no match for provided key".to_string())
        }
    }
    fn cursor_location(&self) -> Option<(usize, usize)> {
        Some((self.tb.cursor.0, self.tb.cursor.1+4))
    }
}















// FILE SYSTEM TAB IMPL
// allows navigation of filesystem to open files
impl FSTab {
    pub fn new(dir: PathBuf) -> FSTab {
        FSTab { line: 0, dir }
    }
}
impl Tab for FSTab {
    fn name(&self) -> String {
        "File Explorer".to_string()
    }
    fn display(&self, width: usize, height: usize) -> String {
        let mut out = String::new();

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

                    out += &format!("{}\n", display);
                }
            }
        }


        out
    }


    fn input(&mut self, key: Key) -> Result<(), String> {
        match key {
            Key::ArrowUp | Key::Char('k') => {
                if self.line > 0 { self.line -= 1; Ok(()) }
                else { Err("".to_string() )}
            },
            Key::ArrowDown | Key::Char('j') => {
                // get num of entries
                let dir_len = if let Ok(mut dir_iter) = read_dir(&self.dir) {
                    dir_iter.count()
                } else { 1 };

                if self.line < dir_len-1 { self.line += 1; Ok(()) }
                else { Err("".to_string() )}
            },
            Key::Enter => {
                // change dir to selected
                if let Ok(mut dir_iter) = read_dir(&self.dir) {
                    let selected = dir_iter.nth(self.line).unwrap().unwrap().path();
                    self.dir = selected;
                }

                Ok(())
            }
            _ => Err("no match for provided key".to_string())
        }

    }

    fn cursor_location(&self) -> Option<(usize, usize)> {
        Some((self.line, 0))
    }

}