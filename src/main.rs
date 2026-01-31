use std::io::{Write, stdout};
use std::env;
use std::path::PathBuf;
use crossterm::cursor::Hide;
use demys::plot::Plot;

use crossterm::{cursor, queue, terminal, QueueableCommand, event, execute};
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode};
use demys::textedit::buffer::TextBuffer;
use demys::textedit::textwindow::TextWindow;
use demys::window::{TestWindow, Window, WindowManager};

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
    // init terminal
    let mut stdout = stdout();
    let _ = crossterm::terminal::enable_raw_mode();
    let _ = execute!(
        stdout,
        EnterAlternateScreen,
        Hide
    );
    let _drop = TuiGuard;



    let mut window_manager = WindowManager::new();




    window_manager.add_window(Box::new(TextWindow::new(TextBuffer::new())));
    //window_manager.add_window(Box::new(TestWindow::default()));


    let mut terminal_dim: Plot = terminal::size().unwrap().into();
    terminal_dim = terminal_dim.transpose();

    window_manager.resize(terminal_dim);
    window_manager.generate_layout();


    window_manager.draw(&mut stdout);

    stdout.flush();


    loop {
        match read().unwrap() {
            Event::Key(KeyEvent { code, kind, modifiers, .. }) => match kind {
                KeyEventKind::Press | KeyEventKind::Repeat => {
                    if let KeyCode::Esc = code { break; }
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
    }


}