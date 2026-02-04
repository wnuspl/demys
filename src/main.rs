use std::collections::VecDeque;
use std::io::{Write, stdout};
use std::env;
use std::ffi::FromVecWithNulError;
use std::path::PathBuf;
use std::time::Duration;
use crossterm::cursor::Hide;
use demys::plot::Plot;

use crossterm::{cursor, queue, terminal, QueueableCommand, event, execute};
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode};
use demys::event::{EventReceiver, Uuid};
use demys::textedit::buffer::TextBuffer;
use demys::window::{TestWindow, Window, WindowManager, WindowRequest};
use demys::window::fswindow::FSWindow;
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


pub enum DemysEvent {
    Sys(Event),
    Request(Uuid, WindowRequest),
}



fn main() {
    let args: Vec<String> = env::args().collect();
    // init terminal
    let mut stdout = stdout();
    let _ = crossterm::terminal::enable_raw_mode();
    let _ = execute!(
        stdout,
        // EnterAlternateScreen,
        Hide
    );
    let _drop = TuiGuard;

    let current_dir = env::current_dir().expect("");

    let mut terminal_dim: Plot = terminal::size().unwrap().into();
    terminal_dim = terminal_dim.transpose();


    let mut window_manager = WindowManager::new();
    window_manager.set_dir(current_dir.clone());
    window_manager.resize(terminal_dim);

    let mut tab = TabWindow::new();
    tab.add_window(Box::new(FSWindow::new(current_dir)));
    window_manager.add_window(Box::new(tab));



    stdout.flush().unwrap();


    let mut events: VecDeque<DemysEvent> = VecDeque::new();
    loop {
        // put window request to queue
        let r = window_manager.collect_events().into_iter().map(|e| {
            DemysEvent::Request(e.0, e.1)
        });
        events.extend(r);

        // put sys events to queue
        if event::poll(Duration::from_millis(0)).unwrap() {
            let sys_event = read().unwrap();
            events.push_back(DemysEvent::Sys(sys_event));
        }



        // match next event
        let e = events.pop_front();
        if e.is_none() { continue; }

        match e.unwrap() {
            DemysEvent::Sys(sys_event) => {
                match sys_event {
                    Event::Key(KeyEvent { kind, code, modifiers, .. }) => {
                        match kind {
                            KeyEventKind::Press => window_manager.input(code, modifiers),
                            _ => ()
                        }
                    },
                    Event::Resize(w, h) => {
                        terminal_dim= terminal::size().unwrap().into();
                        terminal_dim = terminal_dim.transpose();

                        window_manager.resize(terminal_dim);
                        window_manager.draw(&mut stdout);
                    },
                    _ => ()
                }
            }


            DemysEvent::Request(uuid, request) => {
                match request {
                    WindowRequest::Redraw => window_manager.draw_window_uuid(&mut stdout, uuid),
                    WindowRequest::RemoveSelf => {
                        let _ = window_manager.remove_window(uuid);
                    },
                    WindowRequest::AddWindow(window) => {
                        if let Some(window) = window {
                            let mut tab = TabWindow::new();
                            tab.add_window(window);
                            let _ = window_manager.add_window(Box::new(tab));
                        }
                    }
                    WindowRequest::Command(cmd) => {
                        window_manager.run_command(cmd);
                    }
                    _ => ()
                }
            }
        }


        // end actions
        stdout.flush().unwrap();

        if !window_manager.is_active() { break; }
    }
}