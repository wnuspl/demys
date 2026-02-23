use std::collections::{HashMap, VecDeque};
use crate::window::{TestWindow, Window, WindowEvent, WindowRequest};
use std::error::Error;
use std::hash::Hash;
use std::io::{Stdout, Write};
use std::path::PathBuf;
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::{queue, QueueableCommand};
use crossterm::event::{KeyCode, KeyModifiers, ModifierKeyCode};
use crossterm::terminal::{Clear, ClearType};
use crate::event::{EventPoster, EventReceiver, Uuid};
use crate::fswindow::FSWindow;
use crate::window::layout::{BorderSpace, Layout, WindowSpace};
use crate::plot::Plot;
use crate::popup::PopUp;
use crate::style::{Canvas, StyleAttribute, StyledText};
use crate::style::ThemeColor;
use crate::window::command::Command;
use crate::window::tab::TabWindow;
use crate::window::windowcontainer::{OrderedWindowContainer, WindowContainer};

pub struct WindowManager {
    container: OrderedWindowContainer,
    layout: Layout,
    current_dir: PathBuf,
    active: bool,

    require_reset: bool
}


// maps windows to correct shape/position in terminal
impl Window for WindowManager {
    fn init(&mut self, poster: EventPoster<WindowRequest, Uuid>) {
        self.container.set_poster(poster);
    }
    fn event(&mut self, mut event: WindowEvent) {
        if let WindowEvent::Input { key:KeyCode::End, modifiers:KeyModifiers::CONTROL }  = event {
            self.container.post(WindowRequest::RemoveSelfWindow);
            return;
        }

        self.container.distribute_events(&mut event);
        // event may be none now


        // forward events to current
        match event {
            WindowEvent::Input { key:KeyCode::Esc, .. } => {
                self.container.event(WindowEvent::TryQuit);
            }
            WindowEvent::Input { key:KeyCode::Char('l'), modifiers:KeyModifiers::CONTROL, .. } => {
                self.container.cycle_current();
            }
            WindowEvent::Input { key:KeyCode::Char(':'), .. } => {
                self.add_popup(Box::new(
                    Command::default()
                ));
                self.container.post(WindowRequest::Redraw);
            }
            WindowEvent::Input { key:KeyCode::Char('x'), modifiers:KeyModifiers::CONTROL, .. } => {
                if let Some(window) = self.container.get_from_order_mut(self.container.get_current()) {
                    window.event(WindowEvent::TryQuit);
                }
            }


            WindowEvent::None => (),

            event => {
                match event {
                    WindowEvent::Resize(dim) => {
                        self.resize(dim);
                    }
                    _ => ()
                }

                self.container.event(event);
            }
        }

        // self.container.post(WindowRequest::Redraw);
    }
    fn draw(&self, canvas: &mut Canvas) {
        canvas.is_empty(true);
        let window_space = self.layout.get_windows();
        let border_space = self.layout.get_borders();

        for (i, WindowSpace {dim, start}) in window_space.iter().enumerate() {
            if let Some(window) = self.container.get_from_order(i) {
                let mut child_canvas = Canvas::new(*dim);
                window.draw(&mut child_canvas);
                canvas.add_child(child_canvas, *start);
            }
        }

        for border in border_space.iter() {
            let mut border_canvas;
            let text;
            let s;
            let BorderSpace { vertical, length, thickness, start } = border;
            if *vertical {
                border_canvas = Canvas::new(Plot::new(*length, *thickness));
                text = "#".repeat(length * thickness);
                s = start;
            } else {
                border_canvas = Canvas::new(Plot::new(*thickness, *length));
                text = "#".repeat(length*thickness);
                s = start;
            }
            let text = StyledText::new(text)
                .with(StyleAttribute::BgColor(ThemeColor::Black))
                .with(StyleAttribute::Color(ThemeColor::White));
            border_canvas.write_wrap(&text);

            canvas.add_child(border_canvas, *s);
        }

        self.container.draw(canvas);
    }
    fn collect_requests(&mut self) -> Vec<WindowRequest> {
        let requests = self.container.collect_requests();
        for e in requests.iter() {
            if let WindowRequest::AddWindow(..) = e {
                if self.container.window_count() > self.layout.get_windows().len() {
                    self.layout.grid.split_minor(0);
                    self.layout.generate();
                }
            }

            if let WindowRequest::RemoveSelfWindow = e {
                // self.layout.remove_single(0);
                self.layout.generate();
            }

            if let WindowRequest::Command(command) = e {
                if command == "x" {
                    let mut tab = TabWindow::new();
                    tab.add_window(Box::new(FSWindow::new(self.current_dir.clone())));

                    self.container.get_receiver().new_poster().post(
                        WindowRequest::AddWindow(
                            Some(Box::new(tab)
                        )
                    ));
                    self.container.post(WindowRequest::Redraw);
                }
                if command == "qall" {
                    self.event(WindowEvent::TryQuit);
                }
            }
        }

        requests
    }

    fn input_bypass(&self) -> bool {
        self.container.input_bypass()
    }
}

impl WindowContainer for WindowManager {
    fn add_window(&mut self, window: Box<dyn Window>) -> Uuid {
        self.container.add_window(window)
    }
    fn remove_window(&mut self, uuid: Uuid) -> Option<Box<dyn Window>> {
        self.container.remove_window(uuid)
    }
    fn add_popup(&mut self, popup: Box<dyn PopUp>) -> Uuid {
        self.container.add_popup(popup)
    }
    fn remove_popup(&mut self, uuid: Uuid) -> Option<Box<dyn PopUp>> {
        self.container.remove_popup(uuid)
    }

}


impl WindowManager {
    pub fn new() -> Self {
        let mut layout = Layout::new(Plot::new(0,0));
        Self {
            container: OrderedWindowContainer::new(),
            layout,
            current_dir: PathBuf::new(),
            active: true,
            require_reset: false,
        }
    }

    pub fn resize(&mut self, dim: Plot) {
        self.layout.set_dim(dim);
        // propagate
        self.generate_layout();
    }

    fn generate_layout(&mut self) {
        let layout = self.layout.get_windows().clone();
        for (i, WindowSpace {dim, start}) in layout.iter().enumerate() {
            if let Some(window) = self.container.get_from_order_mut(i) {
                window.event(WindowEvent::Resize(*dim));
            }
        }
    }

    pub fn set_dir(&mut self, dir: PathBuf) {
        self.current_dir = dir;
    }
}