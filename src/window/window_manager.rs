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
use crate::window::layout::{BorderSpace, Layout, WindowSpace};
use crate::plot::Plot;
use crate::popup::PopUp;
use crate::style::{Canvas, StyleAttribute, StyledText};
use crate::style::ThemeColor;
use crate::window::command::Command;
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
        if let WindowEvent::Input { key:KeyCode::Esc, .. }  = event {
            self.container.post(WindowRequest::RemoveSelfWindow);
            return;
        }

        self.container.popup_event(&mut event);
        // event may be none now


        // forward events to current
        match event {
            WindowEvent::Input { key:KeyCode::End, .. } => {
                self.container.event(WindowEvent::TryQuit);
            }
            WindowEvent::Input { key:KeyCode::Char('n'), modifiers:KeyModifiers::CONTROL, .. } => {
                self.container.cycle_current();
            }
            WindowEvent::Input { key:KeyCode::Char(':'), .. } => {
                self.add_popup(Box::new(
                    Command::default()
                ));
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
            match border {
                BorderSpace::Horizontal { length, thickness, start} => {
                    border_canvas = Canvas::new(Plot::new(*thickness, *length));
                    text = "#".repeat(length*thickness);
                    s = start;
                }
                BorderSpace::Vertical { length, thickness, start} => {
                    border_canvas = Canvas::new(Plot::new(*length, *thickness));
                    text = "#".repeat(length * thickness);
                    s = start;
                }
            }
            let text = StyledText::new(text)
                .with(StyleAttribute::BgColor(ThemeColor::Black))
                .with(StyleAttribute::Color(ThemeColor::White));
            border_canvas.write_wrap(&text);

            canvas.add_child(border_canvas, *s);
        }

        self.container.draw(canvas);
    }
    fn tick(&mut self) {
        for e in self.container.process_requests() {

            if let WindowRequest::AddWindow(..) = e {
                if self.container.window_count() > self.layout.get_windows().len() {
                    self.layout.grid.split(true);
                    self.layout.generate();
                }
            }

            if let WindowRequest::RemoveSelfWindow = e {
                self.layout.remove_single(0);
                self.layout.generate();
            }
        }

        self.container.tick();
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
        let mut layout = Layout::new();
        layout.generate();
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
        self.layout.generate();
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

    // --------- DRAWING METHODS ------------
    pub fn draw<W: QueueableCommand + Write>(&self, stdout: &mut W) {
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
}