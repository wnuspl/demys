use std::io::{Write, stdout};
use std::env;
use demys::GridPos;
use demys::window_manager::WindowManager;

use crossterm::{cursor, queue, terminal, QueueableCommand, event};
use crossterm::event::{read, Event, KeyCode, KeyEvent};
use crossterm::style::Print;
use crossterm::terminal::enable_raw_mode;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut stdout = stdout();

    crossterm::terminal::enable_raw_mode().unwrap();


    // start with window
    let mut window_manager = WindowManager::new();

    window_manager.layout.split(true);
    //window_manager.layout.split(true);

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
        window_manager.display(&mut stdout, terminal_dim);
        stdout.flush();
    }
}