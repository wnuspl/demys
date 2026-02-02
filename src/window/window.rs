use crossterm::event::{KeyCode, KeyModifiers};
use std::error::Error;
use std::ops::Range;
use crate::plot::Plot;
use crate::style::{Canvas, ThemeColor, StyleAttribute, StyledText};
// holds list of tabs, as well as file system if no tabs are open
// basically just forwards inputs, display requests to correct tab


pub enum WindowRequest {
    Redraw,
    Clear,
    Cursor(Option<Plot>),
    ReplaceWindow(Box<dyn Window>),
    AddWindow(Option<Box<dyn Window>>)
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

        canvas.write(&size);
        canvas.to_next_line();
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