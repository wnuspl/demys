use crate::window::{Window, WindowRequest};
use std::error::Error;
use std::io::Stdout;
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::{queue, QueueableCommand};
use crossterm::event::{KeyCode, KeyModifiers, ModifierKeyCode};
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



    // -------- WINDOW MANAGING METHODS --------
    pub fn add_window(&mut self, window: Box<dyn Window>) {
        self.windows.push(window);
        if self.layout.get_windows().len() < self.windows.len() {
            self.layout.grid.split(true);
        }
    }
    
    pub fn remove_window(&mut self, idx: usize) {
        if self.windows.len() == 1 { return; }

        self.windows.remove(idx);

        if self.focused_window == self.windows.len() {
            self.focused_window = self.windows.len() - 1;
        }

        self.layout.remove_single(idx);
        self.generate_layout();

    }

    // sends input to focused window
    pub fn input(&mut self, key: KeyCode, modifier: KeyModifiers) {
        match (key, modifier) {
            (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
                self.windows[self.focused_window].leave_focus();
                self.focused_window += 1;
                if self.focused_window >= self.windows.len() {
                    self.focused_window = 0;
                }
                self.windows[self.focused_window].on_focus();
            },
            (KeyCode::Char('x'), KeyModifiers::CONTROL) => {
                self.remove_window(self.focused_window);
                self.windows[self.focused_window].on_focus();
            }

            _ => { self.windows[self.focused_window].input(key, modifier); }
        }
    }







    // ---------- UPDATE METHOD -----------

    // Polls all windows and deals with updates
    pub fn update(&mut self, stdout: &mut Stdout) -> Result<(), Box<dyn Error>> {
        let mut replacements = Vec::new();
        let mut redraws = Vec::new();
        let mut clears = Vec::new();
        let mut cursor = Vec::new();
        let mut adds = Vec::new();

        // sort into vecs
        for (i, window) in self.windows.iter_mut().enumerate() {
            for request in window.poll() {
                match request {
                    WindowRequest::Redraw => redraws.push(i),
                    WindowRequest::Clear => clears.push(i),
                    WindowRequest::ReplaceWindow(w) => replacements.push((i, w)),
                    WindowRequest::Cursor(loc) => cursor.push((i, loc)),
                    WindowRequest::AddWindow(w) => adds.push((i, w)),
                }
            }
        }



        // Window replacements
        for (i, w) in replacements {
            self.windows[i] = w;
        }


        // Window additions
        for (i, w) in adds {
            self.add_window(w);

            self.reset_draw(stdout);
        }



        // Clear window
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


        // Redraw window
        for i in redraws {
            self.draw_window(stdout, i);
        }


        // Move cursor
        for (i, loc) in cursor {
            if let Some(loc) = loc {
                if let Some(WindowSpace { start, .. }) = self.layout.get_windows().get(i) {
                    let absolute = loc+*start;
                    queue!(stdout,
                        Show, MoveTo(absolute.col, absolute.row))?;
                }
            } else {
                queue!(stdout,
                    Hide)?;
            }
        }

        Ok(())
    }







    // propogate resize to windows and regenerate layout
    pub fn resize(&mut self, dim: GridPos) {
        self.layout.set_dim(dim);
        self.generate_layout();

        for (i, window_space) in self.layout.get_windows().iter().enumerate() {
            if let Some(w) = self.windows.get_mut(i) {
                w.on_resize(window_space.dim);
            }
        }
    }
    pub fn generate_layout(&mut self) {
        self.layout.generate();
    }






    // --------- DRAWING METHODS ------------


    // Runs window style through style manager and displays to stdout
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


    // complete reset, used to redraw without being called from main
    pub fn reset_draw(&mut self, stdout: &mut Stdout) {
        self.clear(stdout);
        self.generate_layout();
        self.draw(stdout);
    }



    // Draws all borders and windows
    // Called on start, resize, reformat
    pub fn draw(&mut self, stdout: &mut Stdout) -> Result<(), Box<dyn Error>> {
        // draw windows
        for i in 0..self.windows.len() {
            self.draw_window(stdout, i);
        }

        // draw borders
        let border_space = self.layout.get_borders();

        for border in border_space {
            let border_start;
            let border_dim;
            let border = match border {

                // Horizontal border drawing
                BorderSpace::Horizontal { length, thickness, start } => {
                    border_start = *start;
                    border_dim = GridPos::from((*thickness,*length));

                    let mut text = String::new();
                    for _ in 0..*length {
                        text += "─";
                    }

                    vec![StyleItem::Text(text)]
                },


                // Vertical border drawing
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