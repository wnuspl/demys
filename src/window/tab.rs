use std::collections::HashMap;
use std::error::Error;
use std::io::Write;
use std::path::PathBuf;
use crossterm::event::{KeyCode, KeyModifiers};
use crossterm::QueueableCommand;
use crate::event::{EventPoster, EventReceiver, Uuid};
use crate::plot::Plot;
use crate::popup::PopUp;
use crate::style::{Canvas, StyleAttribute, StyledText, ThemeColor};
use crate::window::{Window, WindowEvent, WindowManager, WindowRequest};
use crate::window::command::Command;
use crate::window::layout::{BorderSpace, Layout};
use crate::window::windowcontainer::{WindowContainer};

pub struct TabSettings {
    show_tabs: bool,
}
impl Default for TabSettings {
    fn default() -> Self { Self { show_tabs: true } }
}

pub struct TabWindow {
    windows: HashMap<Uuid, Box<dyn Window>>,
    window_order: Vec<Uuid>,
    event_receiver: EventReceiver<WindowRequest,Uuid>,
    event_poster: Option<EventPoster<WindowRequest,Uuid>>,
    current: usize,
    settings: TabSettings,
    dim: Plot,
}

impl TabWindow {
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
            window_order: Vec::new(),
            event_receiver: EventReceiver::new(),
            event_poster: None,
            current: 0,
            settings: TabSettings::default(),
            dim: Plot::default(),
        }
    }
    fn get_from_order_mut(&mut self, i: usize) -> Option<&mut Box<dyn Window>> {
        if let Some(uuid) = self.window_order.get_mut(i) {
            self.windows.get_mut(uuid)
        } else {
            None
        }
    }
    fn get_from_order(&self, i: usize) -> Option<&Box<dyn Window>> {
        if let Some(uuid) = self.window_order.get(i) {
            self.windows.get(uuid)
        } else {
            None
        }
    }
    fn next_tab(&mut self) {
        self.current = (self.current+1) % self.window_order.len();
    }
}

impl Window for TabWindow {
    fn init(&mut self, poster: EventPoster<WindowRequest, Uuid>) {
        self.event_poster = Some(poster);
    }
    fn event(&mut self, event: WindowEvent) {

        // forward if bypass
        if let Some(window) = self.get_from_order_mut(self.current) {
            if window.input_bypass() {
                window.event(event);
                return;
            }
        }

        // self controls
        match event {
            WindowEvent::Input { key:KeyCode::Tab, .. } => {
                self.next_tab();
            }
            WindowEvent::Input { key:KeyCode::Char('\''), .. } => {
                self.settings.show_tabs = !self.settings.show_tabs;
            }
            WindowEvent::Input { key:KeyCode::Right, modifiers:KeyModifiers::CONTROL, .. } => {
                let uuid =  self.window_order.get(self.current).unwrap();
                let window = self.remove_window(uuid.clone());

                if let Some(window) = window {
                    let mut new_tab = Self::new();
                    new_tab.add_window(window);

                    // send
                    self.event_poster.as_mut().unwrap().post(
                        WindowRequest::AddWindow(
                            Some(Box::new(new_tab))
                        )
                    );

                    self.next_tab();
                }
            }

            WindowEvent::TryQuit => {
                for window in self.windows.values_mut() {
                    window.event(WindowEvent::TryQuit);
                }
            }

            _ => {
                // forward other events
                if let Some(window) = self.get_from_order_mut(self.current) {
                    window.event(event);
                }
            }
        }

        // always redraw?
        self.event_poster.as_mut().unwrap().post(WindowRequest::Redraw);
    }
    fn draw(&self, canvas: &mut Canvas) {
        canvas.is_empty(true);

        // create child canvas
        let child_offset = if self.settings.show_tabs { 1 } else { 0 };

        let child_dim = {
            let mut dim = *canvas.get_dim();
            dim.row -= child_offset;
            dim
        };

        let mut child_canvas = Canvas::new(child_dim);

        // draw to child
        if let Some(window) = self.get_from_order(self.current) {
            window.draw(&mut child_canvas);
        }


        // draw header
        if self.settings.show_tabs {
            let mut header_canvas = Canvas::new(Plot::new(1,canvas.get_dim().col));

            // gray bar across
            header_canvas.set_attribute(
                StyleAttribute::BgColor(ThemeColor::Gray),
                Plot::new(0, 0),
                Plot::new(0, header_canvas.last_col() + 1)).unwrap();


            let tab_space = StyledText::new("|".to_string());

            for (i, uuid) in self.window_order.iter().enumerate() {
                let mut tab = StyledText::new(format!(" {} ", self.windows[uuid].name()));

                // set bg white for current
                if i == self.current { tab = tab.with(StyleAttribute::BgColor(ThemeColor::Background)); }

                header_canvas.write(&tab_space);
                header_canvas.write(&tab);
            }

            header_canvas.write(&tab_space);
            canvas.add_child(header_canvas, Plot::new(0,0));
        }

        // merge
        canvas.add_child(child_canvas, Plot::new(child_offset,0));
    }
    fn tick(&mut self) {
        for (uuid,event) in self.event_receiver.poll() {
            match event {
                WindowRequest::AddWindow(window) => {
                    if let Some(window) = window {
                        self.add_window(window);
                    }
                }
                WindowRequest::RemoveSelfWindow => {
                    self.remove_window(uuid.clone());
                }
                event => {
                    self.event_poster.as_mut().unwrap().post(event);
                }
            }
        }

        for w in self.windows.values_mut() {
            w.tick();
        }

        if self.window_order.len() == 0 {
            self.event_poster.as_mut().unwrap().post(WindowRequest::RemoveSelfWindow);
        }
    }
}

impl WindowContainer for TabWindow {
    fn add_window(&mut self, mut window: Box<dyn Window>) -> Uuid {
        let receiver = self.event_receiver.new_poster();
        let uuid = receiver.get_uuid().clone();
        window.init(receiver);

        self.windows.insert(uuid.clone(), window);
        self.window_order.push(uuid.clone());

        uuid
    }
    fn remove_window(&mut self, uuid: Uuid) -> Option<Box<dyn Window>> {
        let mut order_idx = 0;
        for (i, u) in self.window_order.iter().enumerate() {
            if u == &uuid {
                order_idx = i;
                break;
            }
        }
        self.window_order.remove(order_idx);

        self.windows.remove(&uuid)
    }
    fn add_popup(&mut self, mut popup: Box<dyn PopUp>) -> Uuid {
        self.event_poster.as_mut().unwrap().post(WindowRequest::AddPopup(Some(popup)));
        Uuid(0)
    }
    fn remove_popup(&mut self, uuid: Uuid) -> Option<Box<dyn PopUp>> {
        None
    }
}



