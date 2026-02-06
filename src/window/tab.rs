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
use crate::window::windowcontainer::{OrderedWindowContainer, WindowContainer};

pub struct TabSettings {
    show_tabs: bool,
}
impl Default for TabSettings {
    fn default() -> Self { Self { show_tabs: true } }
}

pub struct TabWindow {
    container: OrderedWindowContainer,
    settings: TabSettings,
    dim: Plot,
}

impl TabWindow {
    pub fn new() -> Self {
        Self {
            container: OrderedWindowContainer::new(),
            settings: TabSettings::default(),
            dim: Plot::default(),
        }
    }

}

impl Window for TabWindow {
    fn init(&mut self, poster: EventPoster<WindowRequest, Uuid>) {
        self.container.set_poster(poster);
    }
    fn event(&mut self, mut event: WindowEvent) {

        self.container.distribute_events(&mut event);
        // event may be none

        // self controls
        match event {
            WindowEvent::Input { key:KeyCode::Tab, .. } => {
                self.container.cycle_current();
            }
            WindowEvent::Input { key:KeyCode::Char('\''), .. } => {
                self.settings.show_tabs = !self.settings.show_tabs;
            }
            WindowEvent::Input { key:KeyCode::Right, modifiers:KeyModifiers::CONTROL, .. } => {
                if let Some(uuid) = self.container.window_order().get(self.container.get_current()) {
                    let window = self.remove_window(uuid.clone());

                    if let Some(window) = window {
                        let mut new_tab = TabWindow::new();
                        new_tab.add_window(window);

                        self.container.post(WindowRequest::AddWindow(
                            Some(Box::new(new_tab))
                        ));

                        self.container.cycle_current()
                    }
                }
            }



            event => {
                self.container.event(event);
            }
        }

        // always redraw?
        self.container.post(WindowRequest::Redraw);
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
        if let Some(window) = self.container.get_from_order(self.container.get_current()) {
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

            // write tab names
            for i in 0..self.container.window_count() {
                if let Some(window) = self.container.get_from_order(i) {
                    let mut tab = StyledText::new(format!(" {} ", window.name()));
                    if i == self.container.get_current() { tab = tab.with(StyleAttribute::BgColor(ThemeColor::Background)); }

                    header_canvas.write(&tab_space);
                    header_canvas.write(&tab);
                }
            }

            // add to main
            header_canvas.write(&tab_space);
            canvas.add_child(header_canvas, Plot::new(0,0));
        }

        // merge
        canvas.add_child(child_canvas, Plot::new(child_offset,0));

        // container
        self.container.draw(canvas);
    }
    fn tick(&mut self) {
        self.container.process_requests();

        self.container.tick();
    }
    fn input_bypass(&self) -> bool {
        self.container.input_bypass()
    }
}

impl WindowContainer for TabWindow {
    fn add_window(&mut self, mut window: Box<dyn Window>) -> Uuid {
        self.container.add_window(window)
    }
    fn remove_window(&mut self, uuid: Uuid) -> Option<Box<dyn Window>> {
        self.container.remove_window(uuid)
    }
    fn add_popup(&mut self, mut popup: Box<dyn PopUp>) -> Uuid {
        self.container.add_popup(popup)
    }
    fn remove_popup(&mut self, uuid: Uuid) -> Option<Box<dyn PopUp>> {
        self.container.remove_popup(uuid)
    }
}



