use demys::window::windowcontainer::{WindowContainer};
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
use crossterm::event::KeyCode::Tab;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, Clear, ClearType};
use demys::event::{EventReceiver, Uuid};
use demys::style::Canvas;
use demys::textedit::buffer::TextBuffer;
use demys::window::{TestWindow, Window, WindowEvent, WindowManager, WindowRequest};
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
        EnterAlternateScreen,
        Hide
    );
    let _drop = TuiGuard;

    let current_dir = env::current_dir().expect("");

    let mut terminal_dim: Plot = terminal::size().unwrap().into();
    terminal_dim = terminal_dim.transpose();


    let mut window_manager = WindowManager::new();
    window_manager.resize(terminal_dim);
    let mut tab_manager = TabWindow::new();
    window_manager.set_dir(current_dir.clone());


    // tab_manager.add_window(Box::new(FSWindow::new(current_dir.clone())));
    // window_manager.add_window(Box::new(tab_manager));


    window_manager.add_window(Box::new(FSWindow::new(current_dir.clone())));



    let mut receiver: EventReceiver<WindowRequest, Uuid> = EventReceiver::new();


    // generic here
    let mut window_container: Box<dyn WindowContainer> = Box::new(window_manager);
    let poster = receiver.new_poster();
    let super_uuid = poster.get_uuid().clone();
    window_container.init(poster);


    stdout.flush().unwrap();


    let mut events: VecDeque<DemysEvent> = VecDeque::new();
    loop {
        // put window request to queue
        let r = receiver.poll().into_iter().map(|e| {
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
                            KeyEventKind::Press => window_container.event(WindowEvent::Input { key:code, modifiers }),
                            _ => ()
                        }
                    },
                    Event::Resize(w, h) => {
                        terminal_dim= terminal::size().unwrap().into();
                        terminal_dim = terminal_dim.transpose();

                        window_container.event(WindowEvent::Resize(terminal_dim));
                        stdout.queue(Clear(ClearType::All)).unwrap();
                    },
                    _ => ()
                }
            }


            DemysEvent::Request(uuid, request) => {
                match request {
                    WindowRequest::Redraw => {
                        let mut canvas = Canvas::new(terminal_dim);
                        window_container.draw(&mut canvas);
                        canvas.queue_write(&mut stdout, Plot::new(0,0));
                    },
                    WindowRequest::RemoveSelf => {
                        break;
                    },
                    WindowRequest::AddWindow(window) => {
                        if let Some(window) = window {
                            let mut tab = TabWindow::new();
                            tab.add_window(window);
                            let _ = window_container.add_window(Box::new(tab));
                        }
                    }
                    WindowRequest::Command(cmd) => {
                        window_container.event(WindowEvent::Command(cmd));
                    }
                    _ => ()
                }
            }
        }

        window_container.tick();

        // end actions
        stdout.flush().unwrap();
    }
}
