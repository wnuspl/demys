mod buffer;

use std::env;
use std::fs::File;
use std::path::Path;
use console::Term;
use console::Key;
use crate::buffer::TextBuffer;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut tb;

    if args.len() < 2 { tb = TextBuffer::new(); }
    else {
        let path_name = args[1].clone();
        let path = Path::new(&path_name);
        // open file
        if let Ok(file) = File::open(path) {
            tb = TextBuffer::from(file);
        } else {
            println!("Error opening file: {}", path_name);
            return;
        }
    }


    // get term handle and write initial file state
    let term = Term::stdout();
    let _ = term.clear_screen();
    let _ = term.write_line(&format!("{}",tb));

    while let Ok(key) = term.read_key_raw() {
        if let Key::Escape = key { break; }
        if let Key::Backspace = key {
            tb.delete(1).is_ok();
        }

        // inserting characters
        if let Key::Enter = key {
            tb.insert("\n");
        }
        if let Key::Char(ch) = key {
            let s = format!("{}", ch);
            tb.insert(&s);
        }

        let _ = term.clear_screen();
        let _ = term.write_line(&format!("{}",tb));

        let _ = term.move_cursor_to(tb.cursor.1, tb.cursor.0);
    }






}