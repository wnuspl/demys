use std::io::{Stdout, Error};
use crate::window::{FSTab, TextTab, Window, WindowRequest};
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::{queue, QueueableCommand};
use crossterm::event::KeyCode;
use crossterm::terminal::{Clear, ClearType};
use crate::GridPos;
use crate::style::{Style, StyleItem};
use crate::layout::*;

pub struct WindowManager {
    pub layout: Layout,
    pub windows: Vec<Box<dyn Window>>,
    pub style: Style,
    focused_window: usize,
}


// maps windows to correct shape/position in terminal


impl WindowManager {
    pub fn new() -> Self {
        Self {
            layout: Layout::new(),
            windows: Vec::new(),
            style: Style::new(),
            focused_window: 0,
        }
    }

    pub fn add_window(&mut self, window: impl Window + 'static) {
        self.windows.push(Box::new(window));
        if self.layout.get_windows().len() < self.windows.len() {
            self.layout.grid.split(true);
        }
    }

    // sends input to focused window
    pub fn input(&mut self, key: KeyCode) -> Result<(),String> {
        if let KeyCode::Tab = key {
            self.windows[self.focused_window].leave_focus();
            self.focused_window += 1;
            if self.focused_window >= self.windows.len() {
                self.focused_window = 0;
            }
            self.windows[self.focused_window].on_focus();
            Ok(())
        } else {
            self.windows[self.focused_window].input(key)
        }
    }







    // Polls all windows, calling appropriate functions to update them
    pub fn update(&mut self, stdout: &mut Stdout) -> Result<(), Error> {
        let mut replacements = Vec::new();
        let mut redraws = Vec::new();
        let mut clears = Vec::new();
        let mut cursor = Vec::new();

        // sort into vecs
        for (i, window) in self.windows.iter_mut().enumerate() {
            for request in window.poll() {
                match request {
                    WindowRequest::Redraw => redraws.push(i),
                    WindowRequest::Clear => clears.push(i),
                    WindowRequest::ReplaceWindow(w) => replacements.push((i, w)),
                    WindowRequest::Cursor(loc) => cursor.push((i, loc))
                }
            }
        }



        for (i, w) in replacements {
            self.windows[i] = w;
        }
        for i in clears {
            if let Some(WindowSpace { start, dim }) = self.layout.get_windows().get(i) {
                let mut clr = Vec::new();
                for _ in 0..dim.row {
                    let mut space = String::new();
                    for _ in 0..dim.col {
                        space += " ";
                    }
                    clr.push(StyleItem::Text(space));
                    clr.push(StyleItem::LineBreak);
                }

                self.style.queue(stdout, clr, *start, *dim);
            }
        }
        for i in redraws {
            self.draw_window(stdout, i);
        }
        for (i, loc) in cursor {
            if let Some(loc) = loc {
                if let Some(WindowSpace { start, .. }) = self.layout.get_windows().get(i) {
                    let absolute = loc+*start;
                    queue!(
                        stdout,
                        Show,
                        MoveTo(absolute.col, absolute.row)
                    )?;
                }
            } else {
                queue!(
                    stdout,
                    Hide
                )?;
            }
        }

        Ok(())
    }





    pub fn generate_layout(&mut self, dim: GridPos) {
        self.layout.generate(dim);
    }






    pub fn draw_window(&self, stdout: &mut Stdout, window_idx: usize) {
        let window = self.windows.get(window_idx);
        if let Some(window) = window {
            let space = self.layout.get_windows().get(window_idx);
            if let Some(WindowSpace { dim, start }) = space {
                self.style.reset(stdout);

                let text = window.style(*dim);
                self.style.queue(stdout, text, *start, *dim);
            }
        }
    }






    // draw all windows/borders
    // only called on start and resize
    pub fn draw(&mut self, stdout: &mut Stdout) -> Result<(),String> {
        let border_space = self.layout.get_borders();
        for i in 0..self.windows.len() {
            self.draw_window(stdout, i);
        }

        for border in border_space {
            let border_start;
            let border_dim;
            let border = match border {
                BorderSpace::Horizontal { length, thickness, start } => {
                    border_start = *start;
                    border_dim = GridPos::from((*thickness,*length));

                    let mut text = String::new();
                    for _ in 0..*length {
                        text += "━";
                    }

                    vec![StyleItem::Text(text)]
                },
                BorderSpace::Vertical { length, thickness, start } => {
                    border_start = *start;
                    border_dim = GridPos::from((*length,*thickness));

                    let mut out = Vec::new();
                    for _ in 0..*length {
                        out.push(StyleItem::Text("│".to_string()));
                        out.push(StyleItem::LineBreak);
                    }

                    out
                }
            };

            self.style.queue(stdout, border, border_start, border_dim);
        }

        Ok(())
    }



    // clears the whole screen
    pub fn clear(&self, stdout: &mut Stdout) {
        let _ = stdout.queue(Clear(ClearType::Purge));
        let _ = stdout.queue(Clear(ClearType::All));
        let _ = stdout.queue(MoveTo(0,0));
    }
}

