use crossterm::event::{KeyCode, KeyModifiers};
use crate::plot::Plot;
use crate::style::{Canvas, StyleAttribute, StyledText, ThemeColor};
use crate::window::{Window, WindowRequest};

#[derive(Default)]
pub struct TabSettings {
    show_tabs: bool,
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

    fn draw(&self, canvas: &mut Canvas) {
        // create child canvas
        let child_canvas_dim = if self.settings.show_tabs {
            *canvas.get_dim() - Plot::new(1,0)
        } else {
            *canvas.get_dim()
        };

        let mut child_canvas = Canvas::new(child_canvas_dim);

        // draw to child
        if let Some(window) = self.windows.get(self.current) {
            window.draw(&mut child_canvas);
        }


        // tab drawing
        let child_offset;
        if self.settings.show_tabs {
            // create header
            let mut tabs = String::new();
            for window in &self.windows {
                tabs += &format!("{} | ", window.name());
            }

            let styled_tabs = StyledText::new(tabs);
            canvas.write_at(&styled_tabs, Plot::new(0,0));
            canvas.set_attribute(
                StyleAttribute::BgColor(ThemeColor::Gray),
                Plot::new(0,0),
                Plot::new(0,canvas.last_col()+1)).unwrap();


            child_offset = Plot::new(1,0);
        } else {
            child_offset = Plot::new(0,0);
        }

        canvas.merge_canvas(child_offset, &child_canvas);

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

                    (KeyCode::Char(';'), _) => {
                        self.settings.show_tabs = !self.settings.show_tabs;
                        self.requests.push(WindowRequest::Redraw);
                    }

                    (KeyCode::Char('l'), KeyModifiers::CONTROL) => {
                        if self.windows.len() == 1 { return; }

                        let window = self.windows.remove(self.current);
                        self.current = (self.current + 1) % self.windows.len();
                        self.requests.push(WindowRequest::AddWindow(Some(window)));
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