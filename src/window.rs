use std::fs;
use std::fs::read_dir;
use std::io::Cursor;
use std::path::PathBuf;
use crossterm::event::{KeyCode, KeyModifiers};
use crate::buffer::TextBuffer;
use crate::GridPos;
use std::error::Error;
use crate::style::{StyleItem, };
// holds list of tabs, as well as file system if no tabs are open
// basically just forwards inputs, display requests to correct tab


pub enum WindowRequest {
    Redraw,
    Clear,
    Cursor(Option<GridPos>),
    ReplaceWindow(Box<dyn Window>),
    AddWindow(Box<dyn Window>)
}


pub trait Window {

    fn name(&self) -> String { String::new() }
    // returns string representation of tab
    fn style(&self, dim: GridPos) -> Vec<StyleItem>;
    fn input(&mut self, key: KeyCode, modifiers: KeyModifiers) {}
    fn on_focus(&mut self) {}
    fn leave_focus(&mut self) {}
    fn on_resize(&mut self, dim: GridPos) {}

    // DO NOT OVERRIDE
    fn poll(&mut self) -> Vec<WindowRequest> { std::mem::take(self.requests()) }
    fn requests(&mut self) -> &mut Vec<WindowRequest>;
}

// adds lines of spaces/fills lines with spaces on right side
pub fn pad(data: &mut Vec<StyleItem>, dim: GridPos) -> Vec<StyleItem> {
    let mut out = Vec::new();
    let mut line_text_len = 0;
    let mut total_lines = 0;
    for (i, item) in std::mem::take(data).into_iter().enumerate() {
        match &item {
            StyleItem::Text(string) => {
                let temp = line_text_len + string.len();
                if temp < dim.col as usize { // don't assign if it goes over
                    line_text_len = temp;
                } else { line_text_len = dim.col as usize; } // avoid adding meaningless spaces to end
            },
            StyleItem::LineBreak => {

                let diff = dim.col as usize - line_text_len;
                if diff > 0 {
                    out.push(StyleItem::Text(" ".repeat(diff)));
                }

                line_text_len = 0;
            },
            _ => ()
        }

        // make sure to add back to vec
        out.push(item);
    }

    // add lines full of spaces at end
    let line_diff = dim.row as usize - total_lines;
    for _ in 0..line_diff {
        out.push(StyleItem::Text(" ".repeat(dim.col as usize)));
        out.push(StyleItem::LineBreak);
    }

    out
}




#[derive(Default)]
pub struct ScrollableData {
    pub screen_rows: u16,
    pub scroll_offset: u16,
    pub total_lines: u16,
    pub scroll_margin: u16
}

// NOTE: need to account for screen sizes less than scroll_margin
pub trait Scrollable {
    fn get_data_mut(&mut self) -> &mut ScrollableData;


    // moves scroll_offset to correct position based on target line
    fn scroll_to(&mut self, line: u16) -> () {
        let data = self.get_data_mut();

        if data.total_lines <= data.screen_rows {
            return;
        }

        let top_line = data.scroll_offset + data.scroll_margin;
        let bot_line = data.scroll_offset + data.screen_rows - data.scroll_margin;

        if line < top_line {
            if line <= data.scroll_margin { data.scroll_offset = 0; return; } // line is too high, can't fit into box, so just go to top

            let correction = (line as i16)-(top_line as i16);
            data.scroll_offset = (data.scroll_offset as i16 + correction) as u16;
        }

        if line > bot_line {
            if line >= data.total_lines-data.scroll_margin { data.scroll_offset = data.total_lines-data.screen_rows; return; } // line is too high, can't fit into box, so just go to top

            let correction = (line as i16)-(bot_line as i16);
            data.scroll_offset = (data.scroll_offset as i16 + correction) as u16;
        }
    }
}

