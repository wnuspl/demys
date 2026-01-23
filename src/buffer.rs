#![allow(unused)]

use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

// Contains text, cursor position, and edit history
// Lowest level of interaction with text, provides functionality for creating/undoing edits
const LINE_BR: &str = "\n";



#[derive(Clone)]
pub struct TextBuffer {
    lines: Vec<String>,
    pub cursor: (usize, usize),
    pub path: Option<PathBuf>,
    saved: bool

}
impl From<PathBuf> for TextBuffer {
    fn from(path: PathBuf) -> Self {
        let file = File::open(&path).unwrap();
        let reader = BufReader::new(file);
        let mut vec = Vec::new();

        for line in reader.lines() {
            if let Ok(line) = line { vec.push(line); }
        }

        // if file is empty
        if vec.iter().len() == 0 { vec = vec!["".to_string()]; }

        TextBuffer {
            lines: vec,
            cursor: (0, 0),
            path: Some(path),
            saved: true
        }
    }
}
impl TextBuffer {
    pub fn new() -> TextBuffer {
        TextBuffer {
            lines: vec!["".to_string()],
            cursor: (0, 0),
            path: None,
            saved: true
        }
    }
    pub fn expand_text(text: &String) -> Vec<String> {
        text.split(LINE_BR).map(String::from).collect()
    }

    pub fn get_lines(&self, start: usize, end: usize) -> &[String] {
        if end > self.lines.len() {
           &self.lines[start..]
        } else {
            &self.lines[start..end]
        }
    }






    // CURSOR METHODS
    // useful methods to move cursor around text buffer and get position

    // Err if line/char doesn't exist
    pub fn cursor_to(&mut self, r: Option<usize>, c: Option<usize>) -> Result<(),String> {
        if let Some(row) = r {
            // oob check
            if self.lines.iter().len() <= row { return Err("".to_string()); }
            self.cursor.0 = row;
        }

        if let Some(col) = c {
            // oob check
            if self.lines[self.cursor.0].len() < col { return Err("".to_string()); }
            self.cursor.1 = col;
        }

        Ok(())
    }

    // Err if line/char doesn't exist
    pub fn cursor_move_by(&mut self, r: Option<isize>, c: Option<isize>) -> Result<(),String> {
        if let Some(row) = r {
            // check that sum isn't negative
            let new_row = (self.cursor.0 as isize)+row;
            if new_row < 0 { return Err("target row is negative".to_string()); }

            let new_row_usize = new_row as usize;

            // oob check
            if self.lines.iter().len() <= new_row_usize { return Err("target row is greater than number of rows".to_string()); }
            self.cursor.0 = new_row_usize;

            // move to end of line if beyond it
            let line_len = self.lines[self.cursor.0].len();
            if line_len < self.cursor.1 {
                self.cursor.1 = line_len;
            }
        }

        if let Some(col) = c {
            // check that sum isn't negative
            let new_col = (self.cursor.1 as isize)+col;
            if new_col < 0 { return Err("".to_string()); }

            let new_col_usize = new_col as usize;

            // oob check
            if self.lines[self.cursor.0].len() < new_col_usize { return Err("".to_string()); }
            self.cursor.1 = new_col_usize;
        }

        Ok(())
    }

    // will always succeed
    pub fn cursor_end_of_line(&mut self) {
        self.cursor.1 = self.lines[self.cursor.0].len();
    }
    pub fn cursor_start_of_line(&mut self) {
        self.cursor.1 = 0;
    }
    pub fn get_cursor_row(&self) -> usize { self.cursor.0 }
    pub fn get_cursor_col(&self) -> usize { self.cursor.1 }









    // EDIT METHODS
    pub fn insert(&mut self, text: &str) -> Result<(),String> {
        let mut edit_line;
        let extra_chars; // chars in edit_line AFTER cursor

        edit_line = self.lines[self.cursor.0].clone();
        extra_chars = edit_line.len() - self.cursor.1;
        edit_line.insert_str(self.cursor.1, text);


        // split into vec
        let expanded = Self::expand_text(&edit_line);
        let edit_line_count = expanded.len();


        // remove and add to edit stack
        let old_line = self.lines.remove(self.cursor.0);

        // insert new text
        self.lines.splice(self.cursor.0..self.cursor.0, expanded);

        let _ = self.cursor_move_by(Some(edit_line_count as isize -1), None);
        self.cursor_end_of_line();
        let _ = self.cursor_move_by(None, Some(extra_chars as isize *-1));

        self.saved = false;

        Ok(())
    }

    // deletes n chars behind cursor
    pub fn delete(&mut self, n: usize) -> Result<(), String> {
        if n == 0 { return Ok(()); }


        if self.cursor == (0,0) { return Err("start of file".to_string()); }

        // remove line
        if self.cursor.1 == 0 {
            let current_line = self.lines.remove(self.cursor.0);

            // move cursor up and to end of line
            self.cursor_move_by(Some(-1), None)?;
            self.cursor_end_of_line();

            // append to previous line
            self.lines[self.cursor.0] += &current_line;

        } else {
            self.cursor_move_by(None, Some(-1))?;
            self.lines[self.cursor.0].remove(self.cursor.1);
        }

        self.saved = false;

        self.delete(n-1)
    }



    pub fn save(&mut self) -> std::io::Result<()> {
        if self.saved { return Ok(()); }

        if let Some(path) = &self.path {
            let mut file = File::create(path)?;
            let text = format!("{}", self);
            let data = text.as_bytes();
            
            file.write_all(data)?;

            self.saved = true;
        }

        Ok(())

    }


}











impl Display for TextBuffer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.lines.join(LINE_BR))
    }
}