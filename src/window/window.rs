use crossterm::event::{KeyCode, KeyModifiers};
use std::error::Error;
use std::ops::Range;
use crate::event::{EventPoster, Uuid};
use crate::plot::Plot;
use crate::popup::PopUp;
use crate::style::{Canvas, ThemeColor, StyleAttribute, StyledText};
use crate::window::WindowManager;
// holds list of tabs, as well as file system if no tabs are open
// basically just forwards inputs, display requests to correct tab


/// Sent to super window (likely WindowManager or TabWindow)
pub enum WindowRequest {
    Redraw,
    Clear,
    Cursor(Option<Plot>),
    RemoveSelf,
    AddWindow(Option<Box<dyn Window>>),
    AddPopup(Option<Box<dyn PopUp>>),
    Command(String),
    None
}

impl Default for WindowRequest {
    fn default() -> WindowRequest {
        WindowRequest::None
    }
}

pub enum WindowEvent {
    Input {
        key: KeyCode,
        modifiers: KeyModifiers
    },
    Resize(Plot),
    Focus,
    Unfocus,
    Command(String),
    TryQuit,
    None
}

impl Default for WindowEvent {
    fn default() -> WindowEvent {
        WindowEvent::None
    }
}


/// Functionality to be a window held by tab or windowmanager
pub trait Window {

    /// Returns string representation of tab.
    /// Used by some super windows.
    fn name(&self) -> String { String::new() }

    /// Request inputs to be sent to directly to self.
    /// Checks managed by super window.
    fn input_bypass(&self) -> bool { false }

    /// Gives a writeable canvas.
    /// Automatically called on
    /// - resize
    fn draw(&self, canvas: &mut Canvas) {}


    /// Is called sometimes
    fn tick(&mut self) {}



    /// Called by super window
    fn event(&mut self, event: WindowEvent) {}
    fn init(&mut self, poster: EventPoster<WindowRequest, Uuid>) {}
}



#[derive(Default)]
pub struct TestWindow {
    content: String,
    command: String,
    dim: Plot,
    focused: bool
}

impl Window for TestWindow {
    fn event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::Command(cmd) => self.command = cmd,
            WindowEvent::Input { key, modifiers } => match key {
                KeyCode::Char(ch) => self.content += &ch.to_string(),
                _ => ()
            }
            WindowEvent::Resize(dim) => self.dim = dim,
            WindowEvent::Focus => self.focused = true,
            WindowEvent::Unfocus => self.focused = false,
            WindowEvent::TryQuit => (),
            WindowEvent::None => ()
        }
    }
    fn draw(&self, canvas: &mut Canvas) {
        let size = StyledText::new(format!("{}", canvas.get_dim()))
            .with(StyleAttribute::Color(ThemeColor::Blue))
            .with(StyleAttribute::BgColor(ThemeColor::Yellow));

        canvas.write(&size);
        canvas.to_next_line();
        canvas.write_wrap(&self.content.clone().into());

        canvas.write_at(&"X".into(), Plot::new(canvas.last_row(),0));
        canvas.write_at(&"X".into(), Plot::new(canvas.last_row(),canvas.last_col()));
        canvas.write_at(&"X".into(), Plot::new(0,canvas.last_col()));
    }
}