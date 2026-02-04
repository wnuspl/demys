use std::collections::HashMap;
use std::error::Error;
use crossterm::event::{KeyCode, KeyModifiers};
use crate::event::{EventPoster, EventReceiver, Uuid};
use crate::plot::Plot;
use crate::style::{Canvas, StyleAttribute, StyledText, ThemeColor};
use crate::window::{Window, WindowEvent, WindowRequest};
use crate::window::command::Command;

pub struct TabSettings {
    show_tabs: bool,
}
impl Default for TabSettings {
    fn default() -> Self { Self { show_tabs: true } }
}

pub struct TabWindow {
    poster: Option<EventPoster<WindowRequest,Uuid>>,
    windows: HashMap<Uuid, Box<dyn Window>>,
    window_order: Vec<Uuid>,
    current: usize,
    settings: TabSettings,
    dim: Plot,
    receiver: EventReceiver<WindowRequest,Uuid>
}

impl TabWindow {
    pub fn new() -> Self {
        Self {
            poster: None,
            windows: HashMap::new(),
            window_order: Vec::new(),
            current: 0,
            settings: TabSettings::default(),
            dim: Plot::new(0,0),
            receiver: EventReceiver::new(),
        }
    }
    pub fn next_tab(&mut self) {
        self.current = (self.current + 1) % self.windows.len();
    }
    pub fn add_window(&mut self, mut window: Box<dyn Window>) {
        // get receiver and uuid
        let r = self.receiver.new_poster();
        let uuid = r.get_uuid().clone();

        // init window
        window.init(r);

        // set window
        self.windows.insert(uuid.clone(), window);
        self.window_order.push(uuid);
    }

    pub fn remove_window(&mut self, uuid: Uuid) {
        self.windows.remove(&uuid);
    }

    // true if found a match
    fn process_input(&mut self, key: KeyCode, modifiers: KeyModifiers) -> bool {
        false
    }

    fn input(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        let uuid = &self.window_order[self.current];
        if let Some(window) = self.windows.get_mut(uuid) {
            if window.input_bypass() {
                window.event(WindowEvent::Input {key, modifiers});
            } else {
                match (key, modifiers) {
                    (KeyCode::Tab, _) => {
                        self.next_tab();
                    }

                    (KeyCode::Char(';'), _) => {
                        self.settings.show_tabs = !self.settings.show_tabs;
                        if self.settings.show_tabs {
                            for (_, window) in self.windows.iter_mut() {
                                // ERROR HERE
                        //        window.event(WindowEvent::Resize(self.dim - Plot::new(1, 0)));
                            }
                        }
                    }

                    (KeyCode::Char(':'), _) => {
                        let command = Command::default();
                        self.add_window(Box::new(command));
                    }

                    (KeyCode::Char('x'), KeyModifiers::CONTROL) => {
                        if self.windows.len() == 1 {
                            //self.requests.push(WindowRequest::RemoveSelf);
                            return;
                        }

                        let _ = self.windows.remove(&uuid);
                        self.next_tab();

                    }
                    _ => window.event(WindowEvent::Input {key, modifiers}),
                }
            }

            self.poster.as_mut().unwrap().post(WindowRequest::Redraw);
        }
    }

}

impl Window for TabWindow {
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
        if let Some(window) = self.windows.get(&self.window_order[self.current]) {
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

            for (i, uuid) in self.window_order.iter().enumerate() {
                let mut tab = StyledText::new(format!(" {} ", self.windows[uuid].name()));

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
        if let Some(window) = self.windows.get(&self.window_order[self.current]) {
            window.input_bypass()
        } else {
            false
        }
    }


    fn event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::Resize(dim) => {
                for (_,window) in self.windows.iter_mut() {
                    self.dim = dim;
                    window.event(WindowEvent::Resize(self.dim));
                }
            },
            WindowEvent::Input {key, modifiers} => {
                self.input(key, modifiers);
            }
            event => {
                if let Some(window) =  self.windows.get_mut(&self.window_order[self.current]) {
                    window.event(event);
                }
            }
        }

        for (uuid, e) in self.receiver.poll() {
            match e {
                WindowRequest::AddWindow(window) => {
                    if let Some(window) = window {
                       self.add_window(window);
                    }
                }
                WindowRequest::RemoveSelf => {
                    self.remove_window(uuid);
                    self.next_tab();
                    // ERROR HERE
                }
                _ => self.poster.as_mut().unwrap().post(e)
            }
        }
    }
    fn init(&mut self, poster: EventPoster<WindowRequest, Uuid>) {
        self.poster = Some(poster);
    }
}