//! Subclass of window that can be written overtop of screen.
//! Typically bypasses standard inputs.

use crossterm::event::{KeyCode, KeyModifiers};
use crate::plot::Plot;
use crate::style::{Canvas, StyleAttribute, ThemeColor};
use crate::window::{Window, WindowRequest};


pub enum PopUpPositionOption {
    Centered(usize),
    NegativeBound(usize),
    PositiveBound(usize),
}

pub struct PopUpPosition {
    pub row: PopUpPositionOption,
    pub col: PopUpPositionOption,
}

pub enum PopUpDimensionOption {
    Fixed(usize),
    Percent(f32),
}

pub struct PopUpDimension {
    pub row: PopUpDimensionOption,
    pub col: PopUpDimensionOption,
}


pub trait PopUp: Window {
    fn position(&self) -> PopUpPosition;
    fn dimension(&self) -> PopUpDimension;
    fn local(&self) -> bool;
}



pub struct Alert {
    content: String,
    requests: Vec<WindowRequest>
}

impl Alert {
    pub fn new(content: String) -> Alert {
        Self {
            content,
            requests: Vec::new()
        }
    }
}

impl Window for Alert {
    fn requests(&mut self) -> &mut Vec<WindowRequest> {
        &mut self.requests
    }

    fn draw(&self, canvas: &mut Canvas) {
        canvas.move_to(Plot::new(2, 5));
       canvas.write(&self.content.clone().into());
        canvas.set_attribute(
            StyleAttribute::BgColor(ThemeColor::Green),
            Plot::new(0,0),
            Plot::new(canvas.last_row(), canvas.last_col()+1)
        );
    }
}

impl PopUp for Alert {
    fn position(&self) -> PopUpPosition {
        PopUpPosition {
            row: PopUpPositionOption::Centered(0),
            col: PopUpPositionOption::Centered(0)
        }
    }
    fn dimension(&self) -> PopUpDimension {
        PopUpDimension {
            row: PopUpDimensionOption::Fixed(5),
            col: PopUpDimensionOption::Fixed(20)
        }
    }
    fn local(&self) -> bool {
        false
    }
}