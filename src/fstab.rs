use std::fs;
use std::fs::read_dir;
use std::io::Cursor;
use std::path::PathBuf;
use crossterm::event::KeyCode;
use crate::GridPos;
use crate::style::{StyleItem, };
use crate::texttab::TextTab;
use crate::window::{Window, WindowRequest};

#[derive(Default)]
struct ScrollableData {
    screen_rows: u16,
    scroll_offset: u16,
    total_lines: u16,
    scroll_margin: u16
}

// NOTE: need to account for screen sizes less than scroll_margin
trait Scrollable {
    fn get_data_mut(&mut self) -> &mut ScrollableData;
    fn scroll_to(&mut self, line: u16) -> () {
        let data = self.get_data_mut();

        let top_line = data.scroll_offset + data.scroll_margin;
        let bot_line = data.scroll_offset + data.screen_rows- data.scroll_margin;

        if line < top_line {
            if line < data.scroll_margin { data.scroll_offset = 0; return; } // line is too high, can't fit into box, so just go to top

            let correction = (line as i16)-(top_line as i16);
            data.scroll_offset = (data.scroll_offset as i16 + correction) as u16;
        }

        if line < bot_line {
            if line > data.total_lines-data.scroll_margin { data.scroll_offset = data.total_lines-data.screen_rows; return; } // line is too high, can't fit into box, so just go to top

            let correction = (line as i16)-(bot_line as i16);
            data.scroll_offset = (data.scroll_offset as i16 + correction) as u16;
        }
    }
}















pub struct FSTab {
    line: u16,
    dir: PathBuf,

    scrollable_data: ScrollableData,




    requests: Vec<WindowRequest>,
}


// FILE SYSTEM TAB IMPL
// allows navigation of filesystem to open files
impl FSTab {
    const SCROLL_MARGIN: u16 = 2;
    pub fn new(dir: PathBuf) -> FSTab {
        FSTab { line: 0, dir, requests: Vec::new(), scrollable_data: ScrollableData::default() }
    }
}





impl Window for FSTab {
    fn name(&self) -> String {
        "File Explorer".to_string()
    }
    fn on_resize(&mut self, dim: GridPos) {
        self.scrollable_data.screen_rows = dim.row;
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