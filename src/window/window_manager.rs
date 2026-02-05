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
use crate::window::windowcontainer::WindowContainer;

pub struct WindowManager {
    windows: HashMap<Uuid, Box<dyn Window>>,
    window_order: Vec<Uuid>,

    popups: VecDeque<Box<dyn PopUp>>,

    event_receiver: EventReceiver<WindowRequest, Uuid>,
    event_poster: Option<EventPoster<WindowRequest, Uuid>>,
    current: usize,
    layout: Layout,
    current_dir: PathBuf,
    active: bool,

    require_reset: bool
}


// maps windows to correct shape/position in terminal
impl Window for WindowManager {
    fn init(&mut self, poster: EventPoster<WindowRequest, Uuid>) {
        self.event_poster = Some(poster);
    }
    fn event(&mut self, event: WindowEvent) {
        if let WindowEvent::Input { key:KeyCode::Esc, .. }  = event {
            self.event_poster.as_mut().unwrap().post(WindowRequest::RemoveSelf);
            return;
        }

        // popup first!
        if let Some(popup) = self.popups.iter_mut().next() {
            popup.event(event);
            self.event_poster.as_mut().unwrap().post(WindowRequest::Redraw);
            return;
        }




        // forward events to current
        match event {
            WindowEvent::Input { key:KeyCode::End, .. } => {
                //self.event_poster.as_mut().unwrap().post(WindowRequest::RemoveSelf);
                for w in self.windows.values_mut() {
                    w.event(WindowEvent::TryQuit);
                }
                return;
            }
            WindowEvent::Input { key:KeyCode::Char('n'), modifiers:KeyModifiers::CONTROL, .. } => {
                self.current = (self.current+1) % self.window_order.len();
            }
            WindowEvent::Input { key:KeyCode::Char(':'), .. } => {
                self.add_popup(Box::new(
                    Command::default()
                ));
            }
            WindowEvent::Resize(dim) => self.resize(dim),

            _ => {
                // forward other events
                if let Some(window) = self.get_from_order_mut(self.current) {
                    window.event(event);
                }
            }
        }

        self.event_poster.as_mut().unwrap().post(WindowRequest::Redraw);
    }
    fn draw(&self, canvas: &mut Canvas) {
        canvas.is_empty(true);
        let window_space = self.layout.get_windows();
        let border_space = self.layout.get_borders();

        for (i, WindowSpace {dim, start}) in window_space.iter().enumerate() {
            if let Some(window) = self.get_from_order(i) {
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


        // draw popups!
        for popup in self.popups.iter() {
            let dim = popup.term_dim(canvas.get_dim());
            let pos = popup.term_pos(canvas.get_dim());

            let mut popup_canvas = Canvas::new(dim);
            popup.draw(&mut popup_canvas);
            canvas.add_child(popup_canvas, pos);
        }
    }
    fn tick(&mut self) {

        for (uuid, event) in self.event_receiver.poll() {
            match event {
                WindowRequest::AddWindow(window) => {
                    if let Some(window) = window {
                        self.add_window(window);
                        self.event_poster.as_mut().unwrap().post(WindowRequest::Redraw);
                    }
                }
                WindowRequest::AddPopup(popup) => {
                    if let Some(popup) = popup {
                        self.add_popup(popup);
                        self.event_poster.as_mut().unwrap().post(WindowRequest::Redraw);
                    }
                }
                WindowRequest::RemoveSelf => {
                    if self.remove_window(uuid.clone()).is_none() {
                        self.remove_popup(uuid);
                    }
                }
                WindowRequest::Command(command) => {
                    if let Some(window) = self.get_from_order_mut(self.current) {
                        window.event(WindowEvent::Command(command));
                    }
                    self.event_poster.as_mut().unwrap().post(WindowRequest::Redraw);
                }
                _ => ()
            }
        }
        for w in self.windows.values_mut() {
            w.tick();
        }

        // if none
        if self.window_order.len() == 0 {
            self.event_poster.as_mut().unwrap().post(WindowRequest::RemoveSelf);
            return;
        }
    }
}

impl WindowContainer for WindowManager {
    fn add_window(&mut self, mut window: Box<dyn Window>) -> Uuid {
        let receiver = self.event_receiver.new_poster();
        let uuid = receiver.get_uuid().clone();
        window.init(receiver);

        self.windows.insert(uuid.clone(), window);
        self.window_order.push(uuid.clone());

        if self.layout.get_windows().len() < self.window_order.len() {
            self.layout.grid.split(true);
            self.layout.generate();
        }

        uuid
    }
    fn remove_window(&mut self, uuid: Uuid) -> Option<Box<dyn Window>> {
        // remove layout
        let mut order_idx = None;
        for (i, u) in self.window_order.iter().enumerate() {
            if u == &uuid {
                self.layout.remove_single(i);
                order_idx = Some(i);
                break;
            }
        }

        if let Some(order_idx) = order_idx {

            self.window_order.remove(order_idx);

            self.generate_layout();

            self.windows.remove(&uuid)
        } else {
            None
        }
    }
    fn add_popup(&mut self, mut popup: Box<dyn PopUp>) {
        let receiver = self.event_receiver.new_poster();
        let uuid = receiver.get_uuid().clone();
        popup.init(receiver);

        self.popups.push_back(popup);
    }
    fn remove_popup(&mut self, uuid: Uuid) -> Option<Box<dyn PopUp>> {
        self.popups.pop_front()
    }
}


impl WindowManager {
    pub fn new() -> Self {
        let mut layout = Layout::new();
        layout.generate();
        Self {
            windows: HashMap::new(),
            window_order: Vec::new(),

            popups: VecDeque::new(),

            event_receiver: EventReceiver::new(),
            event_poster: None,
            layout,
            current: 0,
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
            if let Some(window) = self.get_from_order_mut(i) {
                window.event(WindowEvent::Resize(*dim));
            }
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
    pub fn set_dir(&mut self, dir: PathBuf) {
        self.current_dir = dir;
    }
    pub fn is_active(&self) -> bool {
        self.active
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