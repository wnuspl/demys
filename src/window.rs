use std::fs::{read_dir,ReadDir,DirEntry};
use std::path::PathBuf;
use crate::buffer::TextBuffer;


pub struct Window {
    pub tabs: Vec<TextTab>,
    pub current_tab: Option<usize>,
    fs_tab: FSTab,
    width: usize, // in chars
    height: usize, // in chars
    wrap_text: bool,
}

struct FSTab {
    dir: PathBuf
}

pub struct TextTab {
    tb: TextBuffer,
    name: String
}

trait Tab {
    fn name(&self) -> String;
    fn display(&self) -> String;
}

impl Window {
    pub fn new(width: usize, height: usize, dir: PathBuf) -> Window {
        Window {
            tabs: Vec::new(),
            current_tab: None,
            fs_tab: FSTab::new(dir),
            width,
            height,
            wrap_text: false,
        }
    }


    // calls appropriate tab to get text,
    // wraps or cuts to fit width/height
    pub fn display(&self) -> String {
        // Get data
        let text;
        if let Some(tab) = self.current_tab {
            text = self.tabs[tab].display();
        } else {
            text = self.fs_tab.display();
        }

        let mut out = String::new();

        //let text = self.tabs[self.current_tab.unwrap()].get_lines(0,self.height);
        let mut reached_limit = false;
        let mut line_breaks = 0;
        for line in text.split("\n") {
            line_breaks += 1;


            // wrap/cut string
            let fitted_string: String;
            if self.wrap_text {

                // loop until remaining string can be fit
                let mut split = Vec::new();
                let mut remaining: String = line.to_string();
                while remaining.len() > self.width {
                    split.push(remaining[..self.width].to_string());
                    remaining.replace_range(0..self.width, "");
                    line_breaks += 1;

                    if line_breaks >= self.height {
                        reached_limit = true;
                        break;
                    }
                }

                split.push(remaining);
                fitted_string = split.join("\n   ");
            } else {
                fitted_string = if self.width >= line.len() { line.to_string() } else { line[..self.width].to_string() }
            }

            // add fitted with new line
            out += &format!("{}\n",fitted_string);

            if reached_limit {
                break;
            }
        }

        out
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
    fn display(&self) -> String {
        let raw = format!("{}",self.tb);
        let mut out = String::new();

        // add line numbers
        let mut line_number = 0;
        for line in raw.split("\n") {
            line_number += 1;
            out += &format!("{} | {}\n", line_number, line);
        }

        out
    }
}


// FILE SYSTEM TAB IMPL
// allows navigation of filesystem to open files
impl FSTab {
    pub fn new(dir: PathBuf) -> FSTab {
        FSTab { dir }
    }
}
impl Tab for FSTab {
    fn name(&self) -> String {
        "File Explorer".to_string()
    }
    fn display(&self) -> String {
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
}