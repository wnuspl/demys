use std::alloc::Layout;
use std::error::Error;
use std::mem;
use crate::window::{CharTab, TextTab, Window};
use crate::window::FSTab;
use console::Key;
use console::Term;
use crate::buffer::TextBuffer;
use crate::GridPos;

pub struct WindowManager {
    pub layout: WindowLayout,
    pub windows: Vec<Box<dyn Window>>,
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
        length: usize,
        thickness: usize,
        start: GridPos
    },
    HorizontalBorder {
       length: usize,
       thickness: usize,
       start: GridPos
    }
}


impl WindowManager {
    pub fn new() -> Self {
        Self {
            layout: WindowLayout::new(),
            windows: vec![Box::new(FSTab::new("/".into())),
                          Box::new(TextTab::new(TextBuffer::new(), "hi".to_string())),
                          Box::new(CharTab('X')),
                ],
            focused_window: 0,
        }
    }

    // sends input to focused window
    pub fn input(&mut self, key: Key) -> Result<(),String> {
        if let Key::Tab = key {
            self.focused_window += 1;
            if self.focused_window >= self.windows.len() {
                self.focused_window = 0;
            }
            Ok(())
        } else {
            self.windows[self.focused_window].input(key)
        }
    }

    pub fn display(&self, term: &mut Term, dim: GridPos) {
        let _ = term.clear_screen();
        // get locations
        let layout = self.layout.map_indexes(dim, (0,0).into());


        let mut cursor_location = None;

        let mut windows = self.windows.iter().enumerate();
        for item in layout {
            match item {
                LayoutItem::Window { dim, start } => {
                    if let Some((i, window)) = windows.next() {
                        let text = window.display(dim.col, dim.row);
                        self.display_window(term, text, dim, start);

                        if i == self.focused_window {
                            if let Some(cl) = window.cursor_location() {
                                cursor_location = Some(cl + start);
                            }
                        }
                    }
                },
                LayoutItem::VerticalBorder { length, thickness, start } => {
                    for i in 0..length {
                        let _ = term.move_cursor_to(start.col, start.row + i);
                        print!("|");
                    }
                },
                LayoutItem::HorizontalBorder { length, thickness, start } => {
                    for i in 0..thickness {
                        let _ = term.move_cursor_to(start.col, start.row + i);
                        for j in 0..length {
                            print!("-");
                        }
                    }

                }
            }

        }


        if let Some(cursor_location) = cursor_location {
            let _ = term.show_cursor();
            let _ = term.move_cursor_to(cursor_location.col, cursor_location.row);
        } else {
            let _ = term.hide_cursor();
        }


    }

    fn display_window(&self, term: &mut Term, text: String, dim: GridPos, start: GridPos) {
        let lines = text.split("\n");


        for (i, line) in lines.take(dim.row).enumerate() {
            // move cursor to start of line
            let _ = term.move_cursor_to(start.col, start.row+i);

            // trim chars to avoid overflow
            let trimmed: String = line.chars().take(dim.col).collect();
            print!("{}", trimmed);
        }
    }
}


// all elements in output sum to 1
fn to_dist_vec(vec: &Vec<f32>) -> Vec<f32> {
    let sum = vec.iter().sum::<f32>();
    vec.iter().map(|x| x/sum).collect()
}



impl WindowLayout {
    const BORDER_THICKNESS: usize = 1;
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
                let available_height = dim.row-(body.len()-1)*Self::BORDER_THICKNESS;
                let mut vertical_offset = 0;

                let mut iter = body.iter().zip(heights.iter()).peekable();

                while let Some((layout, height_percent)) = iter.next() {
                    let h = (height_percent*available_height as f32) as usize;

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
                let available_width = dim.col-(body.len()-1)*Self::BORDER_THICKNESS;
                let mut horizontal_offset = 0;

                let mut iter = body.iter().zip(widths.iter()).peekable();

                while let Some((layout, width_percent)) = iter.next() {
                    let w = (width_percent*dim.col as f32) as usize;
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