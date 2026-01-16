use std::error::Error;
use crate::window::Window;
use console::Key;
use console::Term;


pub struct WindowManager {
    layout: WindowLayout,
    pub windows: Vec<Window>,
    focused_window: usize,
}

// Maps windows in vec to locations/sizes in terminal
enum LayoutFormat {
    Horizontal(Vec<usize>),
    Vertical(Vec<usize>),
    Single
}
pub struct WindowLayout {
    format: LayoutFormat,
    width: usize,
    height: usize,
    start: (usize, usize),
}

impl WindowManager {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            layout: WindowLayout::new(width, height, (0,0)),
            windows: vec![Window::new()],
            focused_window: 0,
        }
    }

    // propagates inputs down to Tab::input
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

    pub fn display(&self, term: &mut Term) {
        let _ = term.clear_screen();
        // get locations
        let layout = self.layout.get_positions();

        for (i, pos) in layout.iter().enumerate() {
            self.display_window(term, i, pos.0, pos.1);
        }


        let focused = &self.windows[self.focused_window];
        if let Some(cursor_pos) = focused.cursor_location() {
            let _ = term.show_cursor();
            let _ = term.move_cursor_to(cursor_pos.1+layout[self.focused_window].1.1, cursor_pos.0+layout[self.focused_window].1.0);
        } else {
            let _ = term.hide_cursor();
        }


    }

    pub fn display_window(&self, term: &mut Term, window_idx: usize, dim: (usize, usize), start: (usize, usize)) {
        let (width, height) = dim;
        let window = &self.windows[window_idx];
        for (i, line) in window.display(width,height).split("\n").enumerate() {
            let _ = term.move_cursor_to(start.1, start.0+i);
            print!("{}", line);
        }
    }
}



impl WindowLayout {
    pub fn new(width: usize, height: usize, start: (usize, usize)) -> Self {
        Self { format: LayoutFormat::Vertical(vec![width/2, width/2]), width, height, start }
    }


    // returns Vec<((width, height), (row, col))>
    pub fn get_positions(&self) -> Vec<((usize, usize), (usize, usize))> {
        let mut out: Vec<((usize, usize), (usize, usize))> = Vec::new();

        match &self.format {
            LayoutFormat::Horizontal(heights) => {
                let mut vert_offset = 0;
                for h in heights {
                    out.push((
                        (self.width, *h),
                        (self.start.0+vert_offset, self.start.1)
                    ));
                    vert_offset += h;
                }
            },
            LayoutFormat::Vertical(widths) => {
                let mut hor_offset = 0;
                for w in widths {
                    out.push((
                        (*w, self.height),
                        (self.start.0, self.start.1+hor_offset)
                    ));
                    hor_offset += w;
                }
            },
            LayoutFormat::Single => {
                out.push(((self.width, self.height), self.start));
            }
        }

        out
    }
}