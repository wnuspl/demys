use std::fmt::{Display, Formatter};

// Contains text, cursor position, and edit history
// Lowest level of interaction with text, provides functionality for creating/undoing edits
const LINE_BR: &str = "\n";




pub struct TextBuffer {
    lines: Vec<String>,
    edit_stack: Vec<(usize, String)>,
    pub cursor: (usize, usize),
}

impl TextBuffer {
    pub fn new() -> TextBuffer {
        TextBuffer {
            lines: Vec::new(),
            edit_stack: Vec::new(),
            cursor: (0, 0),
        }
    }

    pub fn expand_text(text: &String) -> Vec<String> {
        text.split(LINE_BR).map(String::from).collect()
    }
    pub fn insert(&mut self, text: &str) {
        let mut edit_line;
        let extra_chars; // chars in edit_line AFTER cursor

        if self.lines.iter().len() == 0 {
            edit_line = text.to_string();
            extra_chars = 0;
        } else {
            edit_line = self.lines[self.cursor.0].clone();
            extra_chars = edit_line.len() - self.cursor.1;
            edit_line.insert_str(self.cursor.1, text);

            // remove and add to edit stack
            let old_line = self.lines.remove(self.cursor.0);
            self.edit_stack.push((self.cursor.0, old_line));
        }

        // split into vec
        let expanded = Self::expand_text(&edit_line);

        // set new cursor pos
        let new_cursor = (
            self.cursor.0 + expanded.len()-1,
            expanded.last().unwrap().len() - extra_chars
        );

        // insert new text and update cursor
        self.lines.splice(self.cursor.0..self.cursor.0, expanded);
        self.cursor = new_cursor;
        //println!("{} cursor: {},{}", text, self.cursor.0, self.cursor.1);
    }
}


impl Display for TextBuffer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.lines.join(LINE_BR))
    }
}