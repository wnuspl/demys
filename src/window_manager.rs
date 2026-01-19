use std::alloc::Layout;
use std::error::Error;
use std::io::{Stdout, Write};
use std::mem;
use std::path::PathBuf;
use crate::window::{CharTab, FSTab, TextTab, Window, WindowRequest};
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::{queue, QueueableCommand};
use crossterm::event::KeyCode;
use crossterm::style::{Attribute, Print, ResetColor, SetAttribute};
use crossterm::terminal::{Clear, ClearType};
use crate::buffer::TextBuffer;
use crate::GridPos;
use crate::style::Style;

pub struct WindowManager {
    pub layout: WindowLayout,
    pub windows: Vec<Box<dyn Window>>,
    pub generated_layout: Vec<LayoutItem>,
    pub style: Style,
    focused_window: usize,
}


// maps windows to correct shape/position in terminal
#[derive(Clone)]
pub enum WindowLayout {
    Horizontal {
        body: Vec<Box<WindowLayout>>,
        heights: Vec<f32>
    },
    Vertical {
        body: Vec<Box<WindowLayout>>,
        widths: Vec<f32>
    },
    Single
}

pub enum LayoutItem {
    Window {
        dim: GridPos,
        start: GridPos
    },
    VerticalBorder {
        length: u16,
        thickness: u16,
        start: GridPos
    },
    HorizontalBorder {
       length: u16,
       thickness: u16,
       start: GridPos
    }
}


impl WindowManager {
    pub fn new() -> Self {
        Self {
            layout: WindowLayout::new(),
            windows: vec![
                          Box::new(TextTab::new(TextBuffer::new(), "hello".to_string()))
                ],
            generated_layout: Vec::new(),
            style: Style::new(),
            focused_window: 0,
        }
    }

    // sends input to focused window
    pub fn input(&mut self, key: KeyCode) -> Result<(),String> {
        if let KeyCode::Tab = key {
            self.focused_window += 1;
            if self.focused_window >= self.windows.len() {
                self.focused_window = 0;
            }
            Ok(())
        } else {
            self.windows[self.focused_window].input(key)
        }
    }

    pub fn update(&mut self) {
        let mut replacements = Vec::new();

        for (i, window) in self.windows.iter_mut().enumerate() {
            if let Some(requests) = window.poll() {
                for request in requests {
                    match request {
                        WindowRequest::Redraw => {},
                        WindowRequest::ReplaceWindow(w) => {
                            replacements.push((i, w));
                        }
                    }
                }
            }
        }

        for (i, w) in replacements {
            self.windows[i] = w;
        }
    }

    pub fn generate_layout(&mut self, dim: GridPos) {
        self.generated_layout = self.layout.map_indexes(dim, (0,0).into());
    }

    pub fn draw(&self, stdout: &mut Stdout) {
        let mut cursor_location = None;
        let mut windows = self.windows.iter().enumerate();
        for item in &self.generated_layout {
            // reset styles
            self.style.reset(stdout);

            match item {
                // Display window
                LayoutItem::Window { dim, start } => {
                    if let Some((i, window)) = windows.next() {
                        let text = window.style(*dim);
                        self.style.queue(stdout, text, *start, *dim);

                        // set cursor if focused
                        if i == self.focused_window {
                            if let Some(cl) = window.cursor_location() { cursor_location = Some(cl + *start); }
                        }
                    }
                },

                // Display borders
                LayoutItem::VerticalBorder { length, thickness, start } => {

                    for i in 0..*length {
                        stdout.queue(MoveTo(start.col, start.row+i));
                        stdout.queue(Print("|"));
                    }
                },
                LayoutItem::HorizontalBorder { length, thickness, start } => {
                }
            }

        }


        if let Some(cursor_location) = cursor_location {
            let _ = queue!(stdout,
                Show,
                MoveTo(cursor_location.col, cursor_location.row)
            );
        } else {
            let _ = stdout.queue(Hide);
        }

    }

    pub fn clear(&self, stdout: &mut Stdout) {
        //clear screen
        let _ = stdout.queue(Clear(ClearType::Purge));
        let _ = stdout.queue(Clear(ClearType::All));
        let _ = stdout.queue(MoveTo(0,0));
    }
}


// all elements in output sum to 1
fn to_dist_vec(vec: &Vec<f32>) -> Vec<f32> {
    let sum = vec.iter().sum::<f32>();
    vec.iter().map(|x| x/sum).collect()
}



impl WindowLayout {
    const BORDER_THICKNESS: u16 = 1;
    pub fn new() -> Self {
        Self::Single
    }
    fn vsplit() -> Self {
        Self::Vertical {
            body: vec![Box::new(WindowLayout::new()), Box::new(WindowLayout::new())],
            widths: vec![0.5, 0.5]
        }
    }
    fn hsplit() -> Self {
        Self::Horizontal {
            body: vec![Box::new(WindowLayout::new()), Box::new(WindowLayout::new())],
            heights: vec![0.5, 0.5]
        }
    }


    // adds another split to layout, if single, defaults to vertical split
    // new window is 1/n size where n is new number of splits
    pub fn split(&mut self, vertical:bool) {
        match self {
            Self::Single => *self = if vertical { WindowLayout::vsplit() } else { WindowLayout::hsplit() },
            Self::Vertical { body, widths } => {
                if vertical {
                    // set width to 1/n
                    let w = 1.0 / (body.len() as f32);
                    widths.push(w);
                    *widths = to_dist_vec(widths);

                    body.push(Box::new(WindowLayout::Single));
                } else {
                    // self becomes inner layout, create 2 way hsplit
                    let inner = mem::replace(self, WindowLayout::hsplit());
                    *self.get_body_mut(0).unwrap() = Box::new(inner);
                }
            },
            Self::Horizontal { body, heights } => {
                if !vertical {
                    // set height to 1/n
                    let h = 1.0 / (body.len() as f32);
                    heights.push(h);
                    *heights = to_dist_vec(heights);

                    body.push(Box::new(WindowLayout::Single));
                } else {
                    // self becomes inner layout, create 2 way vsplit
                    let inner = mem::replace(self, WindowLayout::vsplit());
                    *self.get_body_mut(0).unwrap() = Box::new(inner);
                }
            },
            _ => {}
        }
    }

    pub fn get_body_mut(&mut self, idx: usize) -> Option<&mut Box<WindowLayout>> {
        match self {
            WindowLayout::Single => { None },
            WindowLayout::Vertical { body, .. }
            | WindowLayout::Horizontal { body, .. } => {
                body.get_mut(idx)
            },
            _ => None
        }
    }


    // main function of window layout
    // maps window to physical position in terminal based on size
    // return is Vec<dim, start>
    pub fn map_indexes(&self, dim: GridPos, start: GridPos) -> Vec<LayoutItem> {
        let mut out = Vec::new();
        match &self {
            Self::Single => {
                out.push(LayoutItem::Window{dim, start});
            }

            // Fixed width, variable height
            Self::Horizontal { body, heights } => {
                let available_height = dim.row-(body.len() as u16-1)*Self::BORDER_THICKNESS;
                let mut vertical_offset = 0;

                let mut iter = body.iter().zip(heights.iter()).peekable();

                while let Some((layout, height_percent)) = iter.next() {
                    let h = (height_percent*available_height as f32) as u16;

                    out.append(&mut layout.map_indexes(
                        (h, dim.col).into(),                                // size
                        (start.row+vertical_offset, start.col).into()       // start
                    ));

                    vertical_offset += h;

                    if iter.peek().is_some() {
                        out.push(LayoutItem::HorizontalBorder {
                            thickness: Self::BORDER_THICKNESS,
                            length: dim.col,
                            start: (start.row+vertical_offset, start.col).into()
                        });
                        vertical_offset += Self::BORDER_THICKNESS;
                    }
                }
            },




            // Fixed height, variable width
            Self::Vertical { body, widths } => {
                let available_width = dim.col-(body.len() as u16-1)*Self::BORDER_THICKNESS;
                let mut horizontal_offset = 0;

                let mut iter = body.iter().zip(widths.iter()).peekable();

                while let Some((layout, width_percent)) = iter.next() {
                    let w = (width_percent*available_width as f32) as u16;
                    out.append(&mut layout.map_indexes(
                        (dim.row, w).into(),                                // size
                        (start.row, start.col+horizontal_offset).into()     // start
                    ));
                    horizontal_offset += w;

                    if iter.peek().is_some() {
                        out.push(LayoutItem::VerticalBorder {
                            thickness: Self::BORDER_THICKNESS,
                            length: dim.row,
                            start: (start.row, start.col+horizontal_offset).into()
                        });
                        horizontal_offset += Self::BORDER_THICKNESS;
                    }
                }
            },
            _ => {}
        }
        out
    }
}