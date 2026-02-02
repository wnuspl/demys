use crate::window::{Window, WindowRequest};
use std::error::Error;
use std::io::Stdout;
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::{queue, QueueableCommand};
use crossterm::event::{KeyCode, KeyModifiers, ModifierKeyCode};
use crossterm::terminal::{Clear, ClearType};
use crate::window::layout::{BorderSpace, Layout, WindowSpace};
use crate::plot::Plot;
use crate::style::{Canvas, StyleAttribute, StyledText};
use crate::style::ThemeColor;
use crate::window::tab::TabWindow;

pub struct WindowManager {
    pub layout: Layout,
    pub windows: Vec<Box<dyn Window>>,
    focused_window: usize,
}


// maps windows to correct shape/position in terminal


impl WindowManager {
    pub fn new() -> Self {
        let mut layout = Layout::new();
        layout.generate();
        Self {
            layout,
            windows: Vec::new(),
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
        let mut redraws = Vec::new();
        let mut cursors = Vec::new();
        let mut new_windows = Vec::new();

        // sort into vecs
        for (i, window) in self.windows.iter_mut().enumerate() {
            for request in window.poll() {
                match request {
                    WindowRequest::Redraw => redraws.push(i),
                    WindowRequest::Cursor(loc) => cursors.push((i, loc)),
                    WindowRequest::AddWindow(window) => new_windows.push(window),
                    _ => ()
                }
            }
        }


        // Redraw window
        for i in redraws {
            self.draw_window(stdout, i);
        }

        // Add window
        // Is encased in tab
        for window in new_windows {
            if let Some(window) = window {
                let mut tab = TabWindow::new();
                tab.add_window(window);
                self.add_window(Box::new(tab));
                self.reset_draw(stdout);
            }
        }


        // Move cursor
        for (i, loc) in cursors {
            if let Some(loc) = loc {
                if let Some(WindowSpace { start, .. }) = self.layout.get_windows().get(i) {
                    let absolute = loc+*start;
                    queue!(stdout,
                        Show, MoveTo(absolute.col as u16, absolute.row as u16))?;
                }
            } else {
                queue!(stdout,
                    Hide)?;
            }
        }

        Ok(())
    }







    // propogate resize to windows and regenerate layout
    pub fn resize(&mut self, dim: Plot) {
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
                let mut canvas = Canvas::new(*dim);

                // let window edit canvas
                window.draw(&mut canvas);

                stdout.queue(Hide);
                // write canvas to screen
                canvas.queue_write(stdout, *start);
            }
        }
    }


    // complete reset, used to redraw without being called from main
    pub fn reset_draw(&mut self, stdout: &mut Stdout) {
        self.clear(stdout);
        self.generate_layout();
        self.draw(stdout);
    }


    pub fn draw(&self, stdout: &mut Stdout) {
        for i in 0..self.windows.len() {
            self.draw_window(stdout, i);
        }

        for border_space in self.layout.get_borders() {
            let mut canvas;
            let content;
            let pos;
            match border_space {
                BorderSpace::Vertical {length, thickness, start } => {
                    canvas = Canvas::new(Plot::new(*length, *thickness));
                    content = "|".repeat(length*thickness);
                    pos = start;
                },
                BorderSpace::Horizontal {length, thickness, start} => {
                    canvas = Canvas::new(Plot::new(*length, *thickness));
                    content = "-".repeat(length*thickness);
                    pos = start;
                },
            }
            let _ = canvas.write_wrap(
                &StyledText::new(content)
                    .with(StyleAttribute::Color(ThemeColor::Green))
                    .with(StyleAttribute::BgColor(ThemeColor::Black))
            );

            canvas.queue_write(stdout, *pos);

        }


    }




    // clears the whole screen
    pub fn clear(&self, stdout: &mut Stdout) {
        let _ = stdout.queue(Clear(ClearType::Purge));
        let _ = stdout.queue(Clear(ClearType::All));
        let _ = stdout.queue(MoveTo(0,0));
    }
}