use std::io::{Write, stdout};
use std::env;
use std::path::PathBuf;
use crossterm::cursor::Hide;
use demys::plot::Plot;

use crossterm::{cursor, queue, terminal, QueueableCommand, event, execute};
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode};
use demys::popup::Alert;
use demys::textedit::buffer::TextBuffer;
use demys::textedit::textwindow::TextWindow;
use demys::window::{FSWindow, TestWindow, Window, WindowManager};
use demys::window::tab::TabWindow;

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
    // init terminal
    let mut stdout = stdout();
    let _ = crossterm::terminal::enable_raw_mode();
    let _ = execute!(
        stdout,
        EnterAlternateScreen,
        Hide
    );
    let _drop = TuiGuard;

    let current_dir = env::current_dir().expect("");

    let mut terminal_dim: Plot = terminal::size().unwrap().into();
    terminal_dim = terminal_dim.transpose();


    let mut file_paths: Vec<PathBuf> = Vec::new();
    for p in &args[1..] {
        file_paths.push(p.into());
    }


    let start_tabs: Vec<Box<dyn Window>>;
    if file_paths.len() == 0 {
       start_tabs = vec![
           Box::new(FSWindow::new(current_dir.clone())),
           //Box::new(TestWindow::default()),
       ];
    } else {
        let mut temp: Vec<Box<dyn Window>> = Vec::new();

        // open all files
        for p in file_paths {
            temp.push(Box::new(TextWindow::from_file(p)));
        }

        start_tabs = temp;
    }

    let mut tab = TabWindow::new();
    for t in start_tabs {
        tab.add_window(t);
    }


    let mut window_manager = WindowManager::new();
    window_manager.set_dir(current_dir);

    window_manager.add_window(Box::new(tab));




    window_manager.resize(terminal_dim);
    window_manager.generate_layout();


    window_manager.draw(&mut stdout);

    stdout.flush();


    loop {
        match read().unwrap() {
            Event::Key(KeyEvent { code, kind, modifiers, .. }) => match kind {
                KeyEventKind::Press | KeyEventKind::Repeat => {
                    window_manager.input(code, modifiers);
                },
                _ => {}
            },
            Event::Resize(w, h) => {
                terminal_dim = terminal::size().unwrap().into();
                terminal_dim = terminal_dim.transpose();
                window_manager.resize(terminal_dim);
                window_manager.generate_layout();

                window_manager.draw(&mut stdout);
            },
            _ => ()
        }

        window_manager.update(&mut stdout);


        stdout.flush();

        if !window_manager.is_active() { break; }
    }


}