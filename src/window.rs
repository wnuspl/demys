use std::fs;
use std::fs::read_dir;
use std::io::Cursor;
use std::path::PathBuf;
use crossterm::event::KeyCode;
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
    fn input(&mut self, key: KeyCode) {}
    fn on_focus(&mut self) {}
    fn leave_focus(&mut self) {}
    fn on_resize(&mut self, dim: GridPos) {}

    // DO NOT OVERRIDE
    fn poll(&mut self) -> Vec<WindowRequest> { std::mem::take(self.requests()) }
    fn requests(&mut self) -> &mut Vec<WindowRequest>;
}


// adds spaces to fill unused space in dim
pub fn pad(text: &str, dim: GridPos) -> String {
    let mut out = String::new();
    let mut count = 0;
    for line in text.split("\n") {
        out += &format!("{:<w$}\n", line, w=dim.col as usize);
        count += 1;
    }


    if count < dim.col {
        for _ in 0..(dim.col-count) {
            out += "\n";
        }
    }

    out
}
