use std::path::PathBuf;
use crate::buffer::TextBuffer;


pub struct Window {
    pub tabs: Vec<TextBuffer>,
    pub current_tab: Option<usize>,
    fs_tab: FSTab,
    width: usize, // in chars
    height: usize, // in chars
    wrap_text: bool,
}

struct FSTab {
    dir: PathBuf
}

struct Tab {
    tb: TextBuffer,
    name: String,
}

impl Window {
    pub fn new(width: usize, height: usize) -> Window {
        Window {
            tabs: Vec::new(),
            current_tab: None,
            fs_tab: FSTab::new(),
            width,
            height,
            wrap_text: true,
        }
    }
    pub fn display(&self) -> String {
        let mut out = String::new();



        if let Some(tab) = self.current_tab {

            // if there's a tab to display
            let lines = self.tabs[tab].get_lines(0,self.height);
            let mut reached_limit = false;
            let mut line_number = 0;
            let mut line_breaks = 0;
            for line in lines {
                line_number += 1; line_breaks += 1;


                // wrap/cut string
                let cut_string: String;
                if self.wrap_text {

                    // loop until remaining string can be fit
                    let mut split = Vec::new();
                    let mut remaining = line.clone();
                    while remaining.len() > self.width {
                        split.push(remaining[..self.width].to_string());
                        remaining.replace_range(0..self.width, "");
                        line_breaks += 1;

                        if line_breaks >= self.height { reached_limit = true; break; }
                    }

                    split.push(remaining);
                    cut_string = split.join("\n   ");
                } else {
                    cut_string = if self.width >= line.len() { line.to_string() } else { line[..self.width].to_string() }
                }


                // add line with number to output
                out += &format!("{} | {}\n", line_number, cut_string);

                if reached_limit {
                    break;
                }
            }
        }

        out
    }


}

impl FSTab {
    pub fn new() -> FSTab {
        FSTab {
            dir: PathBuf::from("/")
        }
    }
}