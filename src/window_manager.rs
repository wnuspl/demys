use std::alloc::Layout;
use std::error::Error;
use crate::window::{TextTab, Window};
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


impl WindowManager {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            layout: WindowLayout::new(),
            windows: vec![Box::new(FSTab::new("/".into())), Box::new(TextTab::new(TextBuffer::new(), "hi".to_string())), Box::new(TextTab::new(TextBuffer::new(), "hi".to_string()))],
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

        for (i, pos) in layout.iter().enumerate() {
            self.display_window(term, i, pos.0, pos.1);
        }


        let focused = &self.windows[self.focused_window];
        if let Some(cursor_pos) = focused.cursor_location() {
            let _ = term.show_cursor();
            let _ = term.move_cursor_to(cursor_pos.1+layout[self.focused_window].1.col, cursor_pos.0+layout[self.focused_window].1.row);
        } else {
            let _ = term.hide_cursor();
        }


    }

    fn display_window(&self, term: &mut Term, window_idx: usize, dim: GridPos, start: GridPos) {
        // end if layout has extra spots
        if window_idx >= self.windows.len() { return; }

        let window = &self.windows[window_idx];

        // get text
        let text = window.display(dim.col, dim.row);
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
    pub fn new() -> Self {
        Self::Single
    }
    pub fn vsplit() -> Self {
        Self::Vertical {
            body: vec![Box::new(WindowLayout::new()), Box::new(WindowLayout::new())],
            widths: vec![0.5, 0.5]
        }
    }
    pub fn hsplit() -> Self {
        Self::Horizontal {
            body: vec![Box::new(WindowLayout::new()), Box::new(WindowLayout::new())],
            heights: vec![0.5, 0.5]
        }
    }


    // adds another split to layout, if single, defaults to vertical split
    // new window is 1/n size where n is new number of splits
    pub fn split(&mut self) {
        match self {
            Self::Single => { *self = WindowLayout::vsplit(); },
            Self::Vertical { body, widths } => {
                // set width to 1/n
                let w = 1.0/(body.len() as f32);
                widths.push(w);
                *widths = to_dist_vec(widths);

                body.push(Box::new(WindowLayout::Single));
            },
            Self::Horizontal { body, heights } => {
                // set height to 1/n
                let h = 1.0/(body.len() as f32);
                heights.push(h);
                *heights = to_dist_vec(heights);

                body.push(Box::new(WindowLayout::Single));
            }
        }

    }

    // main function of window layout
    // maps window to physical position in terminal based on size
    // return is Vec<dim, start>
    pub fn map_indexes(&self, dim: GridPos, start: GridPos) -> Vec<(GridPos, GridPos)> {
        let mut out = Vec::new();
        match &self {
            Self::Single => {
                out.push((dim, start));
            }

            // Fixed width, variable height
            Self::Horizontal { body, heights } => {
                let mut vertical_offset = 0;

                for (layout, height_percent) in body.iter().zip(heights.iter()) {
                    let h = (height_percent*dim.row as f32) as usize;
                    out.append(&mut layout.map_indexes(
                        (h, dim.col).into(),                                // size
                        (start.row+vertical_offset, start.col).into()       // start
                    ));
                    vertical_offset += h;
                }
            },


            // Fixed height, variable width
            Self::Vertical { body, widths } => {
                let mut horizontal_offset = 0;
                for (layout, width_percent) in body.iter().zip(widths.iter()) {
                    let w = (width_percent*dim.col as f32) as usize;
                    out.append(&mut layout.map_indexes(
                        (dim.row, w).into(),                                // size
                        (start.row, start.col+horizontal_offset).into()     // start
                    ));
                    horizontal_offset += w;
                }
            }
        }
        out
    }
}