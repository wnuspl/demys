mod buffer;

use std::io::Write;
use console::Term;
use console::Key;
use crate::buffer::TextBuffer;

fn main() {
    let mut tb = TextBuffer::new();
    let mut term = Term::stdout();


    while let Ok(key) = term.read_key_raw() {
        if let Key::Escape = key { break;}
        if let Key::Char(ch) = key {
            let s = format!("{}", ch);
            tb.insert(&s);
            let _ = term.clear_screen();
            let _ = term.write_line(&format!("{}",tb));
        }
    }






}