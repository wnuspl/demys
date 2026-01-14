mod buffer;

use console::Term;
use console::Key;
use crate::buffer::TextBuffer;

fn main() {
    let mut tb = TextBuffer::new();
    let term = Term::stdout();



    while let Ok(key) = term.read_key_raw() {
        if let Key::Escape = key { break; }
        if let Key::Backspace = key {
            if tb.delete(1).is_ok() {
                let _ = term.clear_screen();
                let _ = term.write_line(&format!("{}", tb));
            }
        }

        // inserting characters
        if let Key::Enter = key {
            tb.insert("\n");
            let _ = term.clear_screen();
            let _ = term.write_line(&format!("{}",tb));
        }
        if let Key::Char(ch) = key {
            let s = format!("{}", ch);
            tb.insert(&s);
            let _ = term.clear_screen();
            let _ = term.write_line(&format!("{}",tb));
        }

        let _ = term.move_cursor_to(tb.cursor.1, tb.cursor.0);
    }






}