use std::cmp;
use std::cmp::min;
use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::io::Stdout;
use crossterm::cursor::{MoveTo, Show};
use crossterm::{queue, QueueableCommand};
use crossterm::style::{Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::ClearType;
use crate::plot::Plot;
use crate::style::{StyleAttribute, StyledText};
use crate::window::WindowRequest::Clear;

/// Writeable region that can be written to terminal.
/// Has an immutable size
pub struct Canvas {
    dim: Plot,
    start_style: BTreeMap<usize, Vec<StyleAttribute>>,
    end_style: BTreeMap<usize, Vec<StyleAttribute>>,
    text: String,
    cursor: usize,
    eol: bool,
    show_cursor: bool
}


impl Canvas {
    /// Create a new and empty canvas
    pub fn new(dim: Plot) -> Self {
        // text is filled with spaces
        let text = " ".repeat(dim.col*dim.row).to_string();
        Canvas {
            dim,
            start_style: BTreeMap::new(),
            end_style: BTreeMap::new(),
            text,
            cursor: 0,
            eol: false,
            show_cursor: false
        }
    }

    pub fn get_dim(&self) -> &Plot { &self.dim }
    pub fn expand(&self, loc: usize) -> Plot {
        Plot::new(loc/self.dim.col, loc%self.dim.col)
    }
    pub fn flatten(&self, plot: Plot) -> usize {
        plot.row*self.dim.col + plot.col
    }

    /// Get flattened last space in canvas
    fn get_end(&self) -> usize { self.dim.col*self.dim.row - 1 }



    // CURSOR MOVEMENTS

    /// Moves cursor to specified location.
    /// Err if greater or equal to dimension bounds
    pub fn move_to(&mut self, pos: Plot) -> Result<(), Box<dyn Error>> {
        if pos.row >= self.dim.row { return Err("row out of bounds".into()); }
        if pos.col >= self.dim.col { return Err("col out of bounds".into()); }

        self.cursor = pos.row * self.dim.col + pos.col;

        Ok(())
    }

    /// Moves cursor to beginning of next line.
    /// Err if at last line already
    pub fn to_next_line(&mut self) -> Result<(), Box<dyn Error>> {
        if self.eol { self.eol = false; return Ok(()); }

        let cursor = self.get_cursor();
        self.move_to(Plot::new(cursor.row+1, 0))
    }
    /// Get cursor position as Plot
    pub fn get_cursor(&self) -> Plot {
        Plot::new(self.cursor/self.dim.col, self.cursor%self.dim.col)
    }

    /// Last writeable row
    pub fn last_row(&self) -> usize {
        self.dim.row - 1
    }

    /// Last writeable column
    pub fn last_col(&self) -> usize {
        self.dim.col - 1
    }

    /// Turn on cursor display when canvas is written
    pub fn show_cursor(&mut self, show: bool) {
        self.show_cursor = show;
    }



    // WRITE
    /// Write text to canvas at current cursor position. Cursor moves to next empty spot.
    fn _write(&mut self, text: &StyledText, wrap: bool) {
        if self.eol { self.eol = false; self.cursor += 1; }

        // define range to edit
        let start = self.cursor;
        let mut end = cmp::min(self.cursor+text.len(), self.get_end());

        // if not wrapping
        if !wrap {
            let line = start/self.get_dim().col;
            let temp = (line+1)*self.get_dim().col;
            if end >= temp {
                // line goes over
                self.eol = true;
                end = temp;
            }
        }

        self.text.replace_range(start..end, &text.get_text()[0..end-start]);

        // add style
        for attribute in text.get_attributes() {
            self.set_attribute_flattened(*attribute, start, end);
        }


        self.cursor = end;
    }



    // PUBLIC WRITING METHODS

    /// Write text from cursor.
    pub fn write(&mut self, text: &StyledText) {
        self._write(text, false);
    }
    /// Write text from cursor, moving to next line if extending over last column.
    pub fn write_wrap(&mut self, text: &StyledText) {
        self._write(text, true);
    }
    /// Write text at specified location.
    /// Err if start pos is out of bounds.
    pub fn write_at(&mut self, text: &StyledText, pos: Plot) -> Result<(), Box<dyn Error>> {
        let saved_pos = self.cursor;
        self.move_to(pos)?;
        self._write(text, false);
        self.cursor = saved_pos;
        Ok(())
    }
    /// Write text at specified location, moving to next line if extending over last column.
    /// Err if start pos is out of bounds.
    pub fn write_at_wrap(&mut self, text: &StyledText, pos: Plot) -> Result<(), Box<dyn Error>> {
        let saved_pos = self.cursor;
        self.move_to(pos)?;
        self._write(text, true);
        self.cursor = saved_pos;
        Ok(())
    }


    /// Apply an attribute to region of canvas. (inclusive)..(exclusive)
    pub fn set_attribute(&mut self, attribute: StyleAttribute, start: Plot, end: Plot) -> Result<(), Box<dyn Error>> {
        self.set_attribute_flattened(attribute, self.flatten(start), self.flatten(end))
    }

    fn set_attribute_flattened(&mut self, attribute: StyleAttribute, start: usize, end: usize) -> Result<(), Box<dyn Error>> {
        if start > self.get_end() || end > self.get_end() { return Err("out of bounds".into()); }
        // start bookmark
        if let Some(start_pos) = self.start_style.get_mut(&start) {
            start_pos.push(attribute);
        } else {
            self.start_style.insert(start, vec![attribute]);
        }

        // end bookmark
        if let Some(end_pos) = self.end_style.get_mut(&end) {
            end_pos.push(attribute);
        } else {
            self.end_style.insert(end, vec![attribute]);
        }

        Ok(())
    }



    /// Queues chunk of text body to stdout. Parameters start and end are relative to pos, not at pos
    fn queue_chunk(&mut self, start: usize, end: usize, stdout: &mut Stdout, pos: Plot) {
        let start_line = start/self.dim.col;
        let end_line = end/self.dim.col;



        let mut first_col = start;
        let mut end_col = (start_line+1)*self.dim.col;

        //  loop through lines
        for l in start_line..(end_line+1) {
            if l == end_line {
                end_col = end;
            }

            // get text slice
            let text = &self.text[first_col..end_col];

            // move to first col and l row
            let term_cursor = (
                (first_col%self.dim.col + pos.col) as u16,
                (l+pos.row) as u16
            );

            let _ = queue!(stdout,
                MoveTo(term_cursor.0, term_cursor.1),
                Print(text)
            );

            // update bounds
            first_col = (l+1)*self.dim.col;
            end_col += self.dim.col;
        }
    }

    /// Removes attribute from top of stack and calls apply on next attribute.
    /// Calls reset if last in stack
    fn undo_attribute(stdout: &mut Stdout, variant: usize, attribute_stack: &mut HashMap<usize, Vec<StyleAttribute>>) {
        if let Some(att_vec) = attribute_stack.get_mut(&variant) {
            if let Some(this) = att_vec.pop() {
                // revert if there is old attribute
                if let Some(prev) = att_vec.last() {
                    prev.apply(stdout);
                } else {
                    this.reset(stdout);
                }
            }
        }
    }

    /// Points to stop text chunk to apply styles.
    fn get_breakpoints(&self) -> Vec<usize> {
        let mut break_points = vec![0, self.dim.col*self.dim.row-1];

        break_points.append(&mut self.start_style.keys()
            .map(|x| *x).collect());
        break_points.append(&mut self.end_style.keys()
            .map(|x| *x).collect());

        // order
        break_points.sort();
        break_points.dedup();

        break_points
    }


    /// Write whole canvas to stdout at pos.
    pub fn queue_write(&mut self, stdout: &mut Stdout, pos: Plot) {
        // Marks breakpoints, where style needs to be changed
        // uses queue_chunk to write text in between break points


        // initialize attribute stack
        let mut attribute_stack: HashMap<usize, Vec<StyleAttribute>> = HashMap::new();
        for i in 0..StyleAttribute::COUNT { //
            attribute_stack.insert(i, Vec::new());
        }

        // init iter and prev
        let mut breakpoints = self.get_breakpoints().into_iter();
        let mut prev = breakpoints.next().unwrap();

        for bp in breakpoints {

            // check undo styles
            if let Some(att_vec) = self.end_style.get(&prev) {
                for att in att_vec.iter() {
                    Self::undo_attribute(stdout, usize::from(*att), &mut attribute_stack);
                }
            }

            // check style applications
            if let Some(att_vec) = self.start_style.get(&prev) {
                for att in att_vec.iter() {
                    att.apply(stdout);
                    // add to stack
                    attribute_stack.get_mut(&(usize::from(*att))).unwrap().push(*att);
                }
            }


            // write chunk
            self.queue_chunk(prev, bp, stdout, pos);

            // stdout.queue(crossterm::terminal::Clear(crossterm::terminal::ClearType::All));
            // println!("{},{}",prev,bp);

            prev = bp;
        }

        // in case of failure to reset all, so that they don't bleed over
        for (_, att_vec) in attribute_stack.iter_mut() {
            for att in att_vec.iter_mut() {
                att.reset(stdout);
            }
        }
    }


    /// Copy the content of one canvas to self starting at pos.
    /// Will not wrap content to start of next line if child canvas extends beyond parent bounds.
    pub fn merge_canvas(&mut self, pos: Plot, other: &Canvas) {
        // copy text content
        let max_line_length = self.get_dim().col - pos.col;
        for l in 0..other.get_dim().row {
            // range is line in other canvas
            let range = l*other.get_dim().col..(l+1)*other.get_dim().col;
            // let text: String = other.text[range]
            //     .chars().take(max_line_length).collect(); // take max_line_length
            let text = &other.text[range];

            self.write_at(&text.into(), pos + Plot::new(l,0));
        }


        // map style
        for (start, end) in other.start_style.iter().zip(other.end_style.iter()) {
            let att_list = start.1;
            let start_pos = {
                let (idx, _) = start;
                let line = idx / other.get_dim().col;
                let col = idx % other.get_dim().col;
                Plot::new(line, col) + pos
            };
            let end_pos = {
                let (idx, _) = end;
                let line = idx / other.get_dim().col;
                let col = idx % other.get_dim().col;
                Plot::new(line, col) + pos
            };
            for att in att_list {
                self.set_attribute(*att, start_pos, end_pos);
            }
        }
    }

    /// idk why this is here... replace all text with %
    pub fn block_content(&mut self) {
        self.text = "%".repeat(self.dim.col*self.dim.row);
    }
}




#[cfg(test)]
mod test {
    use crate::style::ThemeColor;
    use super::*;

    #[test]
    fn construction() {
        let dim = Plot::new(20, 40);
        let canvas = Canvas::new(dim);

        assert_eq!(*canvas.get_dim(), dim);
    }

    #[test]
    fn cursor_movement() {
        let dim = Plot::new(20, 40);
        let mut canvas = Canvas::new(dim);

        // starts in 0,0
        assert_eq!(canvas.get_cursor(), Plot::new(0, 0));

        // next line
        canvas.to_next_line().unwrap();
        assert_eq!(canvas.get_cursor(), Plot::new(1, 0));

        // move to valid
        canvas.move_to(Plot::new(5, 10)).unwrap();
        assert_eq!(canvas.get_cursor(), Plot::new(5, 10));
        // move to invalid
        assert!(canvas.move_to(Plot::new(0, dim.col)).is_err());
        assert!(canvas.move_to(Plot::new(dim.row, 0)).is_err());

        // next line resets col
        canvas.to_next_line().unwrap();
        assert_eq!(canvas.get_cursor(), Plot::new(6, 0));
    }

    #[test]
    fn set_style() {
        let dim = Plot::new(20, 40);
        let mut canvas = Canvas::new(dim);

        let _ = canvas.set_attribute(
            StyleAttribute::Color(ThemeColor::Green),
            Plot::new(0,0),
            Plot::new(0, canvas.last_col())
        );
        assert_eq!(canvas.start_style.get(&0).unwrap().len(), 1);
        assert_eq!(canvas.end_style.get(&39).unwrap().len(), 1);

        let _ = canvas.set_attribute(
            StyleAttribute::Color(ThemeColor::Green),
            Plot::new(0,0),
            Plot::new(canvas.last_row(), canvas.last_col())
        );
        assert_eq!(canvas.start_style.get(&0).unwrap().len(), 2);
    }


    #[test]
    fn breakpoints() {
        let dim = Plot::new(20, 40);
        let mut canvas = Canvas::new(dim);

        let text = StyledText::new("hello".to_string())
            .with(StyleAttribute::Color(ThemeColor::Yellow));

        canvas.write(&text);
        canvas.move_to(Plot::new(0, 2)).unwrap();
        canvas.write(&text);
        canvas.move_to(Plot::new(canvas.last_row(), 0)).unwrap();
        canvas.write(&text);
        let bp = canvas.get_breakpoints();

        assert_eq!(bp, vec![
            0,
            2,
            5,
            2+5,
            canvas.last_row()*canvas.get_dim().col,
            canvas.last_row()*canvas.get_dim().col+5,
            canvas.get_end()
        ]);
    }
}