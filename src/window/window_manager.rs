use std::collections::HashMap;
use crate::window::{TestWindow, Window, WindowEvent, WindowRequest};
use std::error::Error;
use std::hash::Hash;
use std::io::{Stdout, Write};
use std::path::PathBuf;
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::{queue, QueueableCommand};
use crossterm::event::{KeyCode, KeyModifiers, ModifierKeyCode};
use crossterm::terminal::{Clear, ClearType};
use crate::event::{EventReceiver, Uuid};
use crate::window::layout::{BorderSpace, Layout, WindowSpace};
use crate::plot::Plot;
use crate::style::{Canvas, StyleAttribute, StyledText};
use crate::style::ThemeColor;

pub struct WindowManager {
    layout: Layout,
    windows: HashMap<Uuid, Box<dyn Window>>,
    window_order: Vec<Uuid>,
    receiver: EventReceiver<WindowRequest, Uuid>,
    focused_window: usize,
    current_dir: PathBuf,
    active: bool,

    require_reset: bool
}


// maps windows to correct shape/position in terminal


impl WindowManager {
    pub fn new() -> Self {
        let mut layout = Layout::new();
        layout.generate();
        Self {
            layout,
            windows: HashMap::new(),
            window_order: Vec::new(),
            receiver: EventReceiver::new(),
            focused_window: 0,
            current_dir: PathBuf::new(),
            active: true,
            require_reset: false,
        }
    }


    pub fn set_dir(&mut self, dir: PathBuf) {
        self.current_dir = dir;
    }


    pub fn collect_events(&mut self) -> Vec<(Uuid, WindowRequest)> {
        self.receiver.poll()
    }

    // -------- WINDOW MANAGING METHODS --------
    /// Create a new window
    pub fn add_window(&mut self, mut window: Box<dyn Window>) -> Result<(), Box<dyn Error>> {
        // get receiver and uuid
        let r = self.receiver.new_poster();
        let uuid = r.get_uuid().clone();

        // init window
        window.init(r);

        // set window
        self.windows.insert(uuid.clone(), window);
        self.window_order.push(uuid);

        Ok(())
    }

    /// Remove window at given uuid
    pub fn remove_window(&mut self, uuid: Uuid) -> Result<(), Box<dyn Error>> {
        if self.windows.remove(&uuid).is_none() {
            Err("window doesn't exist".into())
        } else {
            Ok(())
        }
    }

    pub fn get_window(&self, i: usize) -> Option<&Box<dyn Window>> {
        let uuid = self.window_order.get(i);
        if uuid.is_none() { return None; }

        self.windows.get(uuid.unwrap())
    }
    pub fn get_window_mut(&mut self, i: usize) -> Option<&mut Box<dyn Window>> {
        let uuid = self.window_order.get(i);
        if uuid.is_none() { return None; }

        self.windows.get_mut(uuid.unwrap())
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Quits if all windows accept
    pub fn quit(&mut self) {
        self.active = false;
        // other cleanup?
    }

    /// Quits no matter what
    pub fn force_quit(&mut self) {
        self.active = false;
        // cleanup!!
    }

    pub fn run_command(&mut self, cmd: String) {
        // run personal commands
        match cmd.as_str() {
            "q" => self.quit(),
            "q!" => self.force_quit(),
            "x" => {
                // create explorer
            }
            _ => if let Some(window) = self.get_window_mut(self.focused_window) {
                window.event(WindowEvent::Command(cmd));
            }
        }
    }


    // sends input to focused window
    pub fn input(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        // bypass
        if let Some(window) = self.get_window_mut(self.focused_window) {
            if window.input_bypass() {
                window.event(WindowEvent::Input{key, modifiers});
                return;
            }
        }

        match (key, modifiers) {
            (KeyCode::Esc, _) => self.quit(),
            _ => {
                if let Some(window) = self.get_window_mut(self.focused_window) {
                    window.event(WindowEvent::Input{key, modifiers});
                }
            }
        }
    }









    // propagate resize to windows and regenerate layout
    pub fn resize(&mut self, dim: Plot) {
        self.layout.set_dim(dim);
        self.generate_layout();
        let layout = self.layout.get_windows().clone();

        for (i, window_space) in layout.iter().enumerate() {
            if let Some(window) = self.get_window_mut(i) {
                window.event(WindowEvent::Resize(window_space.dim));
            }
        }
    }
    pub fn generate_layout(&mut self) {
        self.layout.generate();
    }




    // --------- DRAWING METHODS ------------


    // Runs window style through style manager and displays to stdout
    pub fn draw_window<W: QueueableCommand + Write>(&self, stdout: &mut W, i: usize) {
        if let Some(window) = self.get_window(i) {

            // get layout sizes
            let space = self.layout.get_windows().get(i);

            if let Some(WindowSpace { dim, start }) = space {
                let mut canvas = Canvas::new(*dim);

                // let window edit canvas
                window.draw(&mut canvas);

                // write canvas to screen
                canvas.queue_write(stdout, *start);
            }
        }
    }

    pub fn draw_window_uuid<W: QueueableCommand + Write>(&self, stdout: &mut W, uuid: Uuid) {
        for (i, u) in self.window_order.iter().enumerate() {
            if u == &uuid { self.draw_window(stdout, i); }
        }
    }


    // complete reset, used to redraw without being called from main
    pub fn reset_draw<W: QueueableCommand + Write>(&mut self, stdout: &mut W) {
        // self.clear(stdout);
        self.generate_layout();
        self.draw(stdout);
    }


    pub fn draw<W: QueueableCommand + Write>(&self, stdout: &mut W) {
        for i in 0..self.windows.len() {
            self.draw_window(stdout, i);
        }

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



    // clears the whole screen
    pub fn clear<W: QueueableCommand + Write>(&self, stdout: &mut W) {
        let _ = stdout.queue(Clear(ClearType::Purge));
        let _ = stdout.queue(Clear(ClearType::All));
        let _ = stdout.queue(MoveTo(0,0));
    }
}