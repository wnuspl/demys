use std::io::{Write, stdout};
use std::env;
use std::path::PathBuf;
use crossterm::cursor::Hide;
use demys::{GridPos, window};
use demys::window_manager::WindowManager;
use demys::fswindow::FSWindow;
use demys::textwindow::TextWindow;

use crossterm::{cursor, queue, terminal, QueueableCommand, event, execute};
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode};
use demys::tab::TabWindow;
use demys::window::Window;

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
    window_manager.generate_layout();


    let start_tabs: Vec<Box<dyn Window>>;
    // if no files provided
    if file_paths.len() == 0 {
        //window_manager.add_window(FSWindow::new(current_dir));
        start_tabs = vec![Box::new(FSWindow::new(current_dir))];
    } else {
        let mut temp: Vec<Box<dyn Window>> = Vec::new();

        // open all files
        for p in file_paths {
            temp.push(Box::new(TextWindow::from_file(p)));

            //window_manager.add_window(TextWindow::from_file(p));
        }

        start_tabs = temp;
    }

    window_manager.add_window(Box::new(TabWindow::new(start_tabs)));






    
    stdout.flush();

    loop {
        let mut reset = false;
        match read().unwrap() {
            Event::Key(KeyEvent { code, kind, modifiers, .. }) => match kind {
                KeyEventKind::Press | KeyEventKind::Repeat => {
                    if let KeyCode::Esc = code { break; }
                    window_manager.input(code, modifiers);
                },
                _ => {}
            },
            Event::Resize(w, h) => {
                terminal_dim = (h as u16, w as u16).into();


                window_manager.resize(terminal_dim);
                reset = true;

            },
            _ => break
        }
        window_manager.update(&mut stdout);
        if reset {
            window_manager.reset_draw(&mut stdout);
        }
        stdout.flush();
    }

    let _drop = TuiGuard;
}