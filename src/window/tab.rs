use crossterm::event::{KeyCode, KeyModifiers};
use crate::plot::Plot;
use crate::style::Canvas;
use crate::window::{Window, WindowRequest};

pub struct TabWindow {
    requests: Vec<WindowRequest>,
    windows: Vec<Box<dyn Window>>,
    current: usize
}

impl TabWindow {
    pub fn new() -> Self {
        Self {
            requests: Vec::new(),
            windows: Vec::new(),
            current: 0
        }
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


        self.requests.append(&mut local_requests);

        &mut self.requests
    }
    fn draw(&self, canvas: &mut Canvas) {
        if let Some(window) = self.windows.get(self.current) {
            window.draw(canvas);
        }
    }
    fn input(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        if let Some(window) = self.windows.get_mut(self.current) {
            if window.input_bypass() {
                window.input(key, modifiers);
            } else {
                match (key, modifiers) {
                    (KeyCode::Tab, _) => {
                        self.current = (self.current + 1) % self.windows.len();
                        self.requests.push(WindowRequest::Redraw);
                    }
                    _ => window.input(key, modifiers),
                }
            }
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