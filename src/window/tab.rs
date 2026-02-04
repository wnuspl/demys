use std::error::Error;
use crossterm::event::{KeyCode, KeyModifiers};
use crate::plot::Plot;
use crate::style::{Canvas, StyleAttribute, StyledText, ThemeColor};
use crate::window::{Window, WindowRequest};

pub struct TabSettings {
    show_tabs: bool,
}
impl Default for TabSettings {
    fn default() -> Self { Self { show_tabs: true } }
}

pub struct TabWindow {
    requests: Vec<WindowRequest>,
    windows: Vec<Box<dyn Window>>,
    current: usize,
    settings: TabSettings,
}

impl TabWindow {
    pub fn new() -> Self {
        Self {
            requests: Vec::new(),
            windows: Vec::new(),
            current: 0,
            settings: TabSettings::default(),
        }
    }
    pub fn next_tab(&mut self) {
        self.current = (self.current + 1) % self.windows.len();
    }
    pub fn add_window(&mut self, window: Box<dyn Window>) {
        self.windows.push(window);
    }

    // true if found a match
    fn process_input(&mut self, key: KeyCode, modifiers: KeyModifiers) -> bool {
        false
    }

}

impl Window for TabWindow {
    fn requests(&mut self) -> &mut Vec<WindowRequest> {
        let mut local_requests;
        if let Some(window) = self.windows.get_mut(self.current) {
            local_requests = window.poll();
        } else {
            local_requests = Vec::new();
        }


        local_requests.retain_mut(|r| match r {
            WindowRequest::AddWindow(window) => {
                self.add_window(std::mem::take(window).unwrap());
                false
            }
            WindowRequest::Cursor(loc) => {
                if !self.settings.show_tabs { return true; }
                if let Some(loc) = loc {
                    loc.row += 1;
                }
                true
            }
            _ => true
        });



        self.requests.append(&mut local_requests);

        &mut self.requests
    }

    fn try_quit(&mut self) -> Result<(), Box<dyn Error>> {
        for w in self.windows.iter_mut() {
           let r = w.try_quit();
            if r.is_err() { return r; }
        }
        Ok(())
    }

    fn draw(&self, canvas: &mut Canvas) {
        // create child canvas
        let child_offset = if self.settings.show_tabs { 1 } else { 0 };

        let child_dim = {
            let mut dim = *canvas.get_dim();
            dim.row -= child_offset;
            dim
        };

        let mut child_canvas = Canvas::new(child_dim);

        // draw to child
        if let Some(window) = self.windows.get(self.current) {
            window.draw(&mut child_canvas);
        }


        // draw header
        if self.settings.show_tabs {

            // gray bar across
            canvas.set_attribute(
                StyleAttribute::BgColor(ThemeColor::Gray),
                Plot::new(0, 0),
                Plot::new(0, canvas.last_col() + 1)).unwrap();


            let tab_space = StyledText::new("|".to_string());

            for (i, window) in self.windows.iter().enumerate() {
                let mut tab = StyledText::new(format!(" {} ", window.name()));

                // set bg white for current
                if i == self.current { tab = tab.with(StyleAttribute::BgColor(ThemeColor::Background)); }

                canvas.write(&tab_space);
                canvas.write(&tab);
            }

            canvas.write(&tab_space);
        }

        // merge
        canvas.merge_canvas(Plot::new(child_offset, 0), &child_canvas);

    }

    fn input_bypass(&self) -> bool {
        if let Some(window) = self.windows.get(self.current) {
            window.input_bypass()
        } else {
            false
        }
    }

    fn input(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        if let Some(window) = self.windows.get_mut(self.current) {
            if window.input_bypass() {
                window.input(key, modifiers);
            } else {
                match (key, modifiers) {
                    (KeyCode::Tab, _) => {
                        self.next_tab();
                        self.requests.push(WindowRequest::Redraw);
                    }

                    (KeyCode::Char(';'), _) => {
                        self.settings.show_tabs = !self.settings.show_tabs;
                        self.requests.push(WindowRequest::Redraw);
                    }

                    (KeyCode::Char('l'), KeyModifiers::CONTROL) => {
                        if self.windows.len() == 1 { return; }

                        let window = self.windows.remove(self.current);
                        self.next_tab();

                        self.requests.push(WindowRequest::AddWindow(Some(window)));
                    }

                    (KeyCode::Char('x'), KeyModifiers::CONTROL) => {
                        if self.windows.len() == 1 {
                            self.requests.push(WindowRequest::RemoveSelf);
                            return;
                        }

                        let _ = self.windows.remove(self.current);
                        self.next_tab();

                        self.requests.push(WindowRequest::Redraw);
                    }
                    _ => window.input(key, modifiers),
                }
            }
        }
    }

    fn run_command(&mut self, cmd: String) {
        if let Some(window) = self.windows.get_mut(self.current) {
            window.run_command(cmd);
        }
    }

    fn on_focus(&mut self) {
        if let Some(window) = self.windows.get_mut(self.current) {
            window.on_focus();
        }
    }

    fn on_resize(&mut self, dim: Plot) {
        if let Some(window) = self.windows.get_mut(self.current) {
            window.on_resize(dim);
        }
    }
}