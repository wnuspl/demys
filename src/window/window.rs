use crossterm::event::{KeyCode, KeyModifiers};
use std::error::Error;
use crate::plot::Plot;
use crate::style::{Canvas, ThemeColor, StyleAttribute, StyledText};
// holds list of tabs, as well as file system if no tabs are open
// basically just forwards inputs, display requests to correct tab


pub enum WindowRequest {
    Redraw,
    Clear,
    Cursor(Option<Plot>),
    ReplaceWindow(Box<dyn Window>),
    AddWindow(Box<dyn Window>)
}


pub trait Window {

    fn name(&self) -> String { String::new() }
    // returns string representation of tab
    fn input_bypass(&self) -> bool { false }
    fn input(&mut self, key: KeyCode, modifiers: KeyModifiers) {}
    fn on_focus(&mut self) {}
    fn leave_focus(&mut self) {}
    fn on_resize(&mut self, dim: Plot) {}

    fn draw(&self, canvas: &mut Canvas) {}

    // DO NOT OVERRIDE
    fn poll(&mut self) -> Vec<WindowRequest> { std::mem::take(self.requests()) }
    fn requests(&mut self) -> &mut Vec<WindowRequest>;
}



#[derive(Default)]
pub struct ScrollableData {
    pub screen_rows: usize,
    pub scroll_offset: usize,
    pub total_lines: usize,
    pub scroll_margin: usize
}

// NOTE: need to account for screen sizes less than scroll_margin
pub trait Scrollable {
    fn get_data_mut(&mut self) -> &mut ScrollableData;


    // moves scroll_offset to correct position based on target line
    fn scroll_to(&mut self, line: usize) -> () {
        let data = self.get_data_mut();

        if data.total_lines <= data.screen_rows {
            return;
        }

        let top_line = data.scroll_offset + data.scroll_margin;
        let bot_line = data.scroll_offset + data.screen_rows - data.scroll_margin;

        if line < top_line {
            if line <= data.scroll_margin { data.scroll_offset = 0; return; } // line is too high, can't fit into box, so just go to top

            let correction = (line as i16)-(top_line as i16);
            data.scroll_offset = (data.scroll_offset as i16 + correction) as usize;
        }

        if line > bot_line {
            if line >= data.total_lines-data.scroll_margin { data.scroll_offset = data.total_lines-data.screen_rows; return; } // line is too high, can't fit into box, so just go to top

            let correction = (line as i16)-(bot_line as i16);
            data.scroll_offset = (data.scroll_offset as i16 + correction) as usize;
        }
    }
}





#[derive(Default)]
pub struct TestWindow {
    requests: Vec<WindowRequest>,
    content: String
}

impl Window for TestWindow {
    fn requests(&mut self) -> &mut Vec<WindowRequest> { &mut self.requests }

    fn on_resize(&mut self, dim: Plot) {
       self.requests.push(WindowRequest::Redraw);
    }

    fn draw(&self, canvas: &mut Canvas) {
        let size = StyledText::new(format!("{}", canvas.get_dim()))
            .with(StyleAttribute::Color(ThemeColor::Blue))
            .with(StyleAttribute::BgColor(ThemeColor::Yellow));

        canvas.write(&size).unwrap();
        canvas.next_line();
        canvas.write(&self.content.clone().into());

        canvas.show_cursor(true);
    }

    fn input(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        match key {
            KeyCode::Char(ch) => {
                self.content += &ch.to_string();
                self.requests.push(WindowRequest::Redraw);
            }
            _ => ()
        }
    }
}