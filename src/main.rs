mod buffer;
mod window;
mod window_manager;

use std::env;
use std::fs::File;
use std::path::Path;
use console::Term;
use console::Key;
use crate::buffer::TextBuffer;
use crate::window::{TextTab, Window};
use terminal_size::terminal_size;
use crate::window_manager::{WindowLayout, WindowManager};

fn main() {
    let args: Vec<String> = env::args().collect();

    let size = terminal_size();
    let terminal_width = size.unwrap().0.0 as usize;
    let terminal_height = size.unwrap().1.0 as usize;



    // start with window
    let mut window_manager = WindowManager::new(terminal_width, terminal_height, WindowLayout {});

    // get term handle and write initial file state
    let mut term = Term::stdout();
    window_manager.display(&mut term);



    // NOTE:
    // reverts back to direct textbuffer reference here



    while let Ok(key) = term.read_key_raw() {
        if let Key::Escape = key { break; }

        window_manager.input(key);
        window_manager.display(&mut term);
    }

}