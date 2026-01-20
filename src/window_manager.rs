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
use crate::layout::*;

pub struct WindowManager {
    pub layout: Layout,
    pub windows: Vec<Box<dyn Window>>,
    redraws: Vec<usize>,
    pub style: Style,
    focused_window: usize,
}


// maps windows to correct shape/position in terminal


impl WindowManager {
    pub fn new() -> Self {
        Self {
            layout: Layout::new(),
            windows: vec![
                          Box::new(TextTab::new(TextBuffer::new(), "hello".to_string())),
                          Box::new(TextTab::new(TextBuffer::new(), "hello".to_string()))
                ],
            redraws: Vec::new(),
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
            for request in window.poll() {
                match request {
                    WindowRequest::Redraw => {
                        self.redraws.push(i);
                    },
                    WindowRequest::ReplaceWindow(w) => {
                        replacements.push((i, w));
                    }
                }
            }
        }

        for (i, w) in replacements {
            self.windows[i] = w;
        }
    }

    pub fn generate_layout(&mut self, dim: GridPos) {
        self.layout.generate(dim);
    }

    pub fn draw_queued(&mut self, stdout: &mut Stdout) {
        for i in mem::take(&mut self.redraws) {
            self.draw_window(stdout, i);
        }
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

    pub fn draw(&self, stdout: &mut Stdout) {
        //let mut cursor_location = None;
        let mut windows = self.windows.iter();
        let window_space = self.layout.get_windows();
        let border_space = self.layout.get_borders();
        for (WindowSpace { dim, start }, window) in window_space.iter().zip(windows){
            // reset styles
            self.style.reset(stdout);

            let text = window.style(*dim);
            self.style.queue(stdout, text, *start, *dim);

            // set cursor if focused
            /*
            if i == self.focused_window {
                if let Some(cl) = window.cursor_location() { cursor_location = Some(cl + *start); }
            }
            */
        }

        for border in border_space {
            match border {
                BorderSpace::Horizontal { length, thickness, start } => {},
                BorderSpace::Vertical { length, thickness, start } => {}
            }
        }

        /*
        if let Some(cursor_location) = cursor_location {
            let _ = queue!(stdout,
                Show,
                MoveTo(cursor_location.col, cursor_location.row)
            );
        } else {
            let _ = stdout.queue(Hide);
        }
        */
    }

    pub fn clear(&self, stdout: &mut Stdout) {
        //clear screen
        let _ = stdout.queue(Clear(ClearType::Purge));
        let _ = stdout.queue(Clear(ClearType::All));
        let _ = stdout.queue(MoveTo(0,0));
    }
}

