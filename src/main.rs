use std::io::{Write, stdout};
use std::env;
use std::path::PathBuf;
use crossterm::cursor::Hide;
use demys::window::{FSTab, TextTab};
use demys::{GridPos, window};
use demys::window_manager::WindowManager;

use crossterm::{cursor, queue, terminal, QueueableCommand, event, execute};
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode};



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

    let current_dir = env::current_dir().expect("");




    let mut file_paths: Vec<PathBuf> = Vec::new();
    for p in &args[1..] {
        file_paths.push(p.into());
    }


    // init terminal
    let mut stdout = stdout();
    let _ = crossterm::terminal::enable_raw_mode();
    let _ = execute!(
        stdout,
        EnterAlternateScreen,
        Hide
    );

    // terminal dimensions
    let size = crossterm::terminal::size().unwrap();
    let mut terminal_dim= GridPos::from(size).transpose();

    // get window manager
    let mut window_manager = WindowManager::new();
    window_manager.generate_layout(terminal_dim);


    // if no files provided
    if file_paths.len() == 0 {
        window_manager.add_window(FSTab::new(current_dir));
    } else {

        // open all files
        for p in file_paths {
            window_manager.add_window(TextTab::from_file(p));
        }

    }






    
    window_manager.clear(&mut stdout);
    window_manager.draw(&mut stdout);
    stdout.flush();

    loop {
        match read().unwrap() {
            Event::Key(KeyEvent { code, kind, .. }) => match kind {
                KeyEventKind::Press => {
                    if let KeyCode::Esc = code { break; }
                    window_manager.input(code);
                },
                _ => {}
            },
            Event::Resize(w, h) => {
                terminal_dim = (h as u16, w as u16).into();
                window_manager.generate_layout(terminal_dim);
                window_manager.clear(&mut stdout);
                window_manager.draw(&mut stdout);
            },
            _ => break
        }
        window_manager.update(&mut stdout);
        stdout.flush();
    }

    let _drop = TuiGuard;
}