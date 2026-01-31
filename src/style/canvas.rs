use std::cmp;
use std::collections::HashMap;
use std::error::Error;
use std::io::Stdout;
use crossterm::cursor::MoveTo;
use crossterm::{queue, QueueableCommand};
use crossterm::style::{Attribute, Print, ResetColor, SetAttribute, SetForegroundColor};
use crate::Plot;
use crate::style::{StyleAttribute, StyledText};

pub struct Canvas {
    pos: Plot,
    dim: Plot,
    start_style: HashMap<usize, Vec<StyleAttribute>>,
    end_style: HashMap<usize, Vec<StyleAttribute>>,
    text: String,
    cursor: usize
}

impl Canvas {
    pub fn new(pos: Plot, dim: Plot) -> Self {
        // text is filled with spaces
        let text = " ".repeat(dim.col*dim.row).to_string();
        Canvas {
            pos, dim,
            start_style: HashMap::new(),
            end_style: HashMap::new(),
            text,
            cursor: 0
        }
    }

    pub fn get_dim(&self) -> &Plot { &self.dim }

    fn get_end(&self) -> usize { self.dim.col*self.dim.row - 1 }



    // CURSOR MOVEMENTS
    pub fn move_to(&mut self, pos: Plot) -> Result<(), Box<dyn Error>> {
        if pos.row >= self.dim.row { return Err("row out of bounds".into()); }
        if pos.col >= self.dim.col { return Err("col out of bounds".into()); }

        self.cursor = pos.row * self.dim.col + pos.col;

        Ok(())
    }
    // down one line, to first col
    pub fn next_line(&mut self) -> Result<(), Box<dyn Error>> {
        let cursor = self.get_cursor();
        self.move_to(Plot::new(cursor.row+1, 0))
    }
    // convert to plot
    pub fn get_cursor(&self) -> Plot {
        Plot::new(self.cursor/self.dim.col, self.cursor%self.dim.col)
    }


    // WRITE
    pub fn write(&mut self, text: &StyledText) -> Result<(), Box<dyn Error>> {
        //replace text
        let start = self.cursor;
        let end = cmp::min(self.cursor+text.len(), self.get_end());
        self.text.replace_range(start..end, text.get_text());

        // add style
        for attribute in text.get_attributes() {
            self.set_attribute(*attribute, start, end);
        }

        self.cursor = end;

        Ok(())
    }

    pub fn write_at(&mut self, text: &StyledText, pos: Plot) -> Result<(), Box<dyn Error>> {
        let saved_pos = self.cursor;
        self.move_to(pos)?;
        self.write(text)?;
        self.cursor = saved_pos;
        Ok(())
    }

    pub fn set_attribute(&mut self, attribute: StyleAttribute, start: usize, end: usize) {
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
    }



    fn queue_chunk(&mut self, start: usize, end: usize, stdout: &mut Stdout) {
        let start_line = start/self.dim.col;
        let end_line = end/self.dim.col;




        let mut first_col = start;
        let mut end_col = (start_line+1)*self.dim.col;

        //  loop through lines
        for l in start_line..(end_line+1) {

            // get text slice
            let text = &self.text[first_col..end_col];

            // move to first col and l row
            let term_cursor = (
                (first_col%self.dim.col + self.pos.col) as u16,
                (l+self.pos.row) as u16
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
    fn apply_attribute(stdout: &mut Stdout, attribute: StyleAttribute) {
        match attribute {
            StyleAttribute::Color(color) => {
                let _ = stdout.queue(
                    SetForegroundColor(color.into())
                );
            }
            StyleAttribute::Bold(bold) => {
                let _ = stdout.queue(
                    if bold {
                        SetAttribute(Attribute::Bold)
                    } else {
                        SetAttribute(Attribute::NormalIntensity)
                    }
                );
            }
        }
    }

    // undoes top of stack, reapply what's underneath
    fn undo_attribute(stdout: &mut Stdout, variant: usize, attribute_stack: &mut HashMap<usize, Vec<StyleAttribute>>) {
        if let Some(att_vec) = attribute_stack.get_mut(&variant) {
            if let Some(this) = att_vec.pop() {
                // revert if there is old attribute
                if let Some(prev) = att_vec.last() {
                    Self::apply_attribute(stdout, *prev);
                } else {
                    // else, reset
                    let _ = match this {
                        StyleAttribute::Color(_) => {
                            stdout.queue(ResetColor)
                        }
                        StyleAttribute::Bold(_) => {
                            stdout.queue(SetAttribute(Attribute::NormalIntensity))
                        }
                    };
                }
            }
        }
    }


    // queues whole canvas write to stdout
    pub fn queue_write(&mut self, stdout: &mut Stdout) {
        // Marks breakpoints, where style needs to be changed
        // uses queue_chunk to write text in between break points
        let mut break_points = vec![0, self.dim.col*self.dim.row-1];

        break_points.append(&mut self.start_style.keys()
            .map(|x| *x).collect());
        break_points.append(&mut self.end_style.keys()
            .map(|x| *x).collect());

        // order
        break_points.sort();
        break_points.dedup();



        // initialize attribute stack
        let mut attribute_stack: HashMap<usize, Vec<StyleAttribute>> = HashMap::new();
        for i in 0..StyleAttribute::COUNT { //
            attribute_stack.insert(i, Vec::new());
        }

        // init iter and prev
        let mut break_points = break_points.into_iter();
        let mut prev = break_points.next().unwrap();

        for bp in break_points {

            // check undo styles
            if let Some(att_vec) = self.end_style.get(&prev) {
                for att in att_vec.iter() {
                    Self::undo_attribute(stdout, usize::from(*att), &mut attribute_stack);
                }
            }

            // check style applications
            if let Some(att_vec) = self.start_style.get(&prev) {
                for att in att_vec.iter() {
                    Self::apply_attribute(stdout, *att);
                    // add to stack
                    attribute_stack.get_mut(&(usize::from(*att))).unwrap().push(*att);
                }
            }


            // write chunk
            self.queue_chunk(prev, bp, stdout);

            prev = bp;
        }
    }
}




#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn construction() {
        let dim = Plot::new(20, 40);
        let canvas = Canvas::new(Plot::new(0, 0),  dim);

        assert_eq!(*canvas.get_dim(), dim);
    }

    #[test]
    fn cursor_movement() {
        let dim = Plot::new(20, 40);
        let mut canvas = Canvas::new(Plot::new(0, 0),  dim);

        // starts in 0,0
        assert_eq!(canvas.get_cursor(), Plot::new(0, 0));

        // next line
        canvas.next_line().unwrap();
        assert_eq!(canvas.get_cursor(), Plot::new(1, 0));

        // move to valid
        canvas.move_to(Plot::new(5, 10)).unwrap();
        assert_eq!(canvas.get_cursor(), Plot::new(5, 10));
        // move to invalid
        assert!(canvas.move_to(Plot::new(0, dim.col)).is_err());
        assert!(canvas.move_to(Plot::new(dim.row, 0)).is_err());

        // next line resets col
        canvas.next_line().unwrap();
        assert_eq!(canvas.get_cursor(), Plot::new(6, 0));
    }
}