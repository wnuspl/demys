use std::io::{Write, stdout};
use std::env;
use demys::GridPos;
use demys::window_manager::WindowManager;

use crossterm::{cursor, queue, terminal, QueueableCommand, event, execute};
use crossterm::event::{read, Event, KeyCode, KeyEvent};
use crossterm::style::Print;
use crossterm::terminal::{enable_raw_mode, LeaveAlternateScreen};



struct TuiGuard;

impl Drop for TuiGuard {
    fn drop(&mut self) {
        let mut stdout = stdout();
        // Best effort cleanup; ignore errors because we're in Drop.
        let _ = terminal::disable_raw_mode();
        let _ = execute!(stdout, cursor::Show, LeaveAlternateScreen);
        let _ = stdout.flush();
    }
}



fn main() {
    let args: Vec<String> = env::args().collect();

    let mut stdout = stdout();

    crossterm::terminal::enable_raw_mode().unwrap();


    // start with window
    let mut window_manager = WindowManager::new();

    window_manager.layout.split(true);

    // get term handle and write initial file state
    let size = crossterm::terminal::size().unwrap();
    let mut terminal_dim= GridPos::from(size).transpose();
    window_manager.display(&mut stdout, terminal_dim);

    stdout.flush();

    loop {
        match read().unwrap() {
            Event::Key(KeyEvent { code, .. }) => {
                if let KeyCode::Esc = code { break; }

                window_manager.input(code);
            }
            Event::Resize(w, h) => {
                terminal_dim = (h as u16, w as u16).into();
            }
            _ => break
        }
        window_manager.update();
        window_manager.display(&mut stdout, terminal_dim);
        stdout.flush();
    }

    let _drop = TuiGuard;
}