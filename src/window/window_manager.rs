use crate::window::{FSWindow, TestWindow, Window, WindowRequest};
use std::error::Error;
use std::io::{Stdout, Write};
use std::path::PathBuf;
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::{queue, QueueableCommand};
use crossterm::event::{KeyCode, KeyModifiers, ModifierKeyCode};
use crossterm::terminal::{Clear, ClearType};
use crate::window::layout::{BorderSpace, Layout, WindowSpace};
use crate::plot::Plot;
use crate::popup::{PopUp, PopUpDimensionOption, PopUpPosition, PopUpPositionOption};
use crate::style::{Canvas, StyleAttribute, StyledText};
use crate::style::ThemeColor;
use crate::window::cmddisplay::CmdDisplay;
use crate::window::tab::TabWindow;

pub struct WindowManager {
    layout: Layout,
    windows: Vec<Box<dyn Window>>,
    focused_window: usize,
    current_dir: PathBuf,
    pub cmd_display: Option<Box<CmdDisplay>>,
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
            windows: Vec::new(),
            focused_window: 0,
            current_dir: PathBuf::new(),
            cmd_display: None,
            active: true,
            require_reset: false,
        }
    }


    pub fn set_dir(&mut self, dir: PathBuf) {
        self.current_dir = dir;
    }



    // -------- WINDOW MANAGING METHODS --------
    pub fn add_window(&mut self, window: Box<dyn Window>) -> Result<(), Box<dyn Error>> {
        self.windows.push(window);
        if self.layout.get_windows().len() < self.windows.len() {
            self.layout.grid.split(true);
        }

        self.generate_layout();

        Ok(())
    }
    
    pub fn remove_window(&mut self, idx: usize) -> Result<(), Box<dyn Error>> {
        if self.windows.len() == 1 { return Err("only one window".into()); }

        self.windows.remove(idx);

        if self.focused_window == self.windows.len() {
            self.focused_window = self.windows.len() - 1;
        }

        self.layout.remove_single(idx);

        self.generate_layout();
        Ok(())
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Quits if all windows accept
    pub fn quit(&mut self) {
        let mut response = Vec::new();
        for w in self.windows.iter_mut() {
            let r = w.try_quit();
            if r.is_err() { response.push(r); }
        }

        if response.len() == 0 { self.active = false; }
        // other cleanup?
    }

    /// Quits no matter what
    pub fn force_quit(&mut self) {
        self.active = false;
        // cleanup!!
    }

    fn run_command(&mut self, cmd: String) {
        // run personal commands
        match cmd.as_str() {
            "q" => self.quit(),
            "q!" => self.force_quit(),
            "x" => {
                self.add_window(Box::new({
                    let mut tab = TabWindow::new();
                    tab.add_window(Box::new(FSWindow::new(self.current_dir.clone())));
                    tab
                }));
                self.focused_window = self.windows.len() - 1;
                self.require_reset = true;
            }
            _ => if let Some(window) = self.windows.get_mut(self.focused_window) {
                window.run_command(cmd);
            }
        }
    }


    // sends input to focused window
    pub fn input(&mut self, key: KeyCode, modifier: KeyModifiers) {
        // accumulate command
        if let Some(cmd_display) = &mut self.cmd_display {
            let cmd = &mut cmd_display.cmd;
            match (key, modifier) {
                (KeyCode::Esc, _) | (KeyCode::Char('['), KeyModifiers::CONTROL) => {
                    self.cmd_display = None;
                    // todo: figure out how to redraw underlying windows
                }
                (KeyCode::Char(ch), _) => {
                    *cmd += &ch.to_string();
                },
                (KeyCode::Backspace, _) => if cmd.len() > 0 {
                    cmd.remove(cmd.len()-1);
                }
                (KeyCode::Enter, _) => {
                    let cmd = self.cmd_display.take().unwrap().cmd;
                    self.run_command(cmd);
                }
                _ => ()
            }

            return;
        }


        // bypass
        if let Some(window) = self.windows.get_mut(self.focused_window) {
            if window.input_bypass() {
                window.input(key, modifier);
                return;
            }
        }

        match (key, modifier) {
            (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
                self.windows[self.focused_window].leave_focus();
                self.focused_window += 1;
                if self.focused_window >= self.windows.len() {
                    self.focused_window = 0;
                }
                self.windows[self.focused_window].on_focus();
            },
            (KeyCode::Char(':'), _) => {
                self.cmd_display = Some(Box::new(CmdDisplay::default()));
            }
            _ => { self.windows[self.focused_window].input(key, modifier); }
        }
    }







    // ---------- UPDATE METHOD -----------

    // Polls all windows and deals with updates
    pub fn update<W: QueueableCommand + Write>(&mut self, stdout: &mut W) -> Result<(), Box<dyn Error>> {
        let mut redraws = Vec::new();
        let mut cursors = Vec::new();
        let mut new_windows = Vec::new();
        let mut removes = Vec::new();

        // sort into vecs
        for (i, mut window) in self.windows.iter_mut().enumerate() {
            for request in window.poll() {
                match request {
                    WindowRequest::Redraw => redraws.push(i),
                    WindowRequest::RemoveSelf => removes.push(i),
                    WindowRequest::Cursor(loc) => cursors.push((i, loc)),
                    WindowRequest::AddWindow(window) => new_windows.push(window),
                    _ => ()
                }
            }
        }


        // Redraw window
        for i in redraws {
            self.draw_window(stdout, i);
        }

        // Add window
        // Is encased in tab
        for window in new_windows {
            if let Some(window) = window {
                let mut tab = TabWindow::new();
                tab.add_window(window);
                self.add_window(Box::new(tab));
                self.require_reset = true;
            }
        }

        for i in removes {
            if self.remove_window(i).is_ok() {
                self.require_reset = true;
            }
        }


        // Move cursor
        for (i, loc) in cursors {
            if let Some(loc) = loc {
                if let Some(WindowSpace { start, .. }) = self.layout.get_windows().get(i) {
                    let absolute = loc + *start;
                    queue!(stdout,
                        Show, MoveTo(absolute.col as u16, absolute.row as u16))?;
                }
            } else {
                queue!(stdout,
                    Hide)?;
            }
        }


        // fix!!
        if let Some(cmd_display) = &self.cmd_display {
            self.draw_popup(stdout, cmd_display);
        }

        if self.require_reset {
            self.reset_draw(stdout);
            self.require_reset = false;
        }

        Ok(())
    }



    // propogate resize to windows and regenerate layout
    pub fn resize(&mut self, dim: Plot) {
        self.layout.set_dim(dim);
        self.generate_layout();

        for (i, window_space) in self.layout.get_windows().iter().enumerate() {
            if let Some(w) = self.windows.get_mut(i) {
                w.on_resize(window_space.dim);
            }
        }
    }
    pub fn generate_layout(&mut self) {
        self.layout.generate();
    }






    // --------- DRAWING METHODS ------------


    // Runs window style through style manager and displays to stdout
    pub fn draw_window<W: QueueableCommand + Write>(&self, stdout: &mut W, window_idx: usize) {
        let window = self.windows.get(window_idx);
        if let Some(window) = window {
            let space = self.layout.get_windows().get(window_idx);
            if let Some(WindowSpace { dim, start }) = space {
                let mut canvas = Canvas::new(*dim);

                // let window edit canvas
                window.draw(&mut canvas);

                stdout.queue(Hide);
                // write canvas to screen
                canvas.queue_write(stdout, *start);
            }
        }
    }

    fn to_term_pos(avail: usize, pos: PopUpPositionOption) -> usize {
        match pos {
            PopUpPositionOption::Centered(offset) => offset + (avail/2),
            PopUpPositionOption::NegativeBound(offset) => offset,
            PopUpPositionOption::PositiveBound(offset) => avail - offset,
        }
    }
    fn to_term_dim(avail: usize, pos: PopUpDimensionOption) -> usize {
        match pos {
            PopUpDimensionOption::Fixed(n) => n,
            PopUpDimensionOption::Percent(p) => ((avail as f32)*p) as usize
        }
    }

    pub fn draw_popup<W: QueueableCommand + Write>(&self, stdout: &mut W, popup: &Box<CmdDisplay>) {
        let pos = popup.position();
        let dim = popup.dimension();

        let total_dim = self.layout.get_dim();


        let term_dim = Plot::new(
            Self::to_term_dim(total_dim.row, dim.row),
            Self::to_term_dim(total_dim.col, dim.col)
        );

        let term_pos = Plot::new(
            Self::to_term_pos(total_dim.row, pos.row) - term_dim.row/2,
            Self::to_term_pos(total_dim.col, pos.col) - term_dim.col/2,
        );

        let mut canvas = Canvas::new(term_dim);
        popup.draw(&mut canvas);

        canvas.queue_write(stdout, term_pos);
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