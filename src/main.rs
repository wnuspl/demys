use std::io::Write;
use console::Term;
use console::Key;

fn main() {
    let mut term = Term::stdout();


    while let Ok(key) = term.read_key_raw() {
        if let Key::Char(ch) = key {
            if ch == 'q' { break; }
            let s = format!("{}", ch);
            let _ = term.write(s.as_bytes());
        }
    }
}