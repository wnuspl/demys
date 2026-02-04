use crossterm::event::{KeyCode, KeyModifiers};
use std::error::Error;
use std::ops::Range;
use crate::plot::Plot;
use crate::style::{Canvas, ThemeColor, StyleAttribute, StyledText};
// holds list of tabs, as well as file system if no tabs are open
// basically just forwards inputs, display requests to correct tab


/// Sent to super window (likely WindowManager or TabWindow)
pub enum WindowRequest {
    Redraw,
    Clear,
    Cursor(Option<Plot>),
    RemoveSelf,
    AddWindow(Option<Box<dyn Window>>)
}


/// Functionality to be a window held by tab or windowmanager
pub trait Window {

    /// Give access to requests, will be consumed on update
    fn requests(&mut self) -> &mut Vec<WindowRequest>;

    fn name(&self) -> String { String::new() }
    // returns string representation of tab

    /// Request inputs to be sent to directly to self.
    fn input_bypass(&self) -> bool { false }
    fn input(&mut self, key: KeyCode, modifiers: KeyModifiers) {}
    /// Called when window is set to the active (focused) window
    fn on_focus(&mut self) {}
    /// Called when leaving focus
    fn leave_focus(&mut self) {}
    /// Only guaranteed to be called on terminal resize and layout adjustment
    fn on_resize(&mut self, dim: Plot) {}

    /// Gives a writeable canvas
    fn draw(&self, canvas: &mut Canvas) {}

    fn run_command(&mut self, cmd: String) {}

    /// Return Err if program should wait to quit
    fn try_quit(&mut self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }


    /// Used by super windows, [do not override]
    fn poll(&mut self) -> Vec<WindowRequest> { std::mem::take(self.requests()) }
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
        canvas.write_wrap(&self.content.clone().into());

        canvas.write_at(&"X".into(), Plot::new(canvas.last_row(),0));
        canvas.write_at(&"S".into(), Plot::new(canvas.last_row(),canvas.last_col()-1));
        canvas.write_at(&"X".into(), Plot::new(canvas.last_row(),canvas.last_col()));
        canvas.write_at(&"X".into(), Plot::new(0,canvas.last_col()));
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

    fn run_command(&mut self, cmd: String) {
        self.content += &cmd;
        self.requests.push(WindowRequest::Redraw);
    }
}