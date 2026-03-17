use std::{cmp, fs};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::path::PathBuf;
use std::slice::Iter;
use crate::plot::Plot;
use crate::textedit::fixed_char;
use crate::textedit::operation::{TextBufferOperation, TBOperationError, InsertString};

struct Metrics {
    pub length: usize,
    new_lines_order: Vec<usize>,
}

impl Metrics {
    pub fn set_linebreak(&mut self, mut gap_index: usize) {
        let i = self.new_lines_order.iter().position(|lb| {
            lb > &gap_index
        });
        if let Some(i) = i {
            self.new_lines_order.insert(i, gap_index);
        } else {
            self.new_lines_order.push(gap_index);
        }
    }
    pub fn remove_linebreak(&mut self, gap_index: usize) {
        let i = self.new_lines_order.iter().position(|lb| {
            lb == &gap_index
        });
        if let Some(i) = i {
            self.new_lines_order.remove(i);
        }
    }

    pub fn get_new_line_order(&self) -> &Vec<usize> {
        &self.new_lines_order
    }
}

pub struct TextBuffer {
    content: Vec<fixed_char>,
    cursor: usize,
    gap_end: usize,
    operations: Vec<Box<dyn TextBufferOperation>>,
    metrics: Metrics,
}

impl TextBuffer {
    const DEFAULT_GAP_SIZE: usize = 200;
    pub fn new() -> TextBuffer {
        TextBuffer {
            content: vec![' ' as fixed_char; TextBuffer::DEFAULT_GAP_SIZE],
            cursor: 0,
            gap_end: Self::DEFAULT_GAP_SIZE,
            operations: Vec::new(),
            metrics: Metrics {
                length: 0,
                new_lines_order: Vec::new(),
            }
        }
    }
    pub fn apply(&mut self, mut operation: Box<dyn TextBufferOperation>) -> Result<(), TBOperationError> {
        let result = operation.apply(self);

        if let Err(error) = result {
            match self.resolve_operation_error(error) {
                Ok(()) => self.apply(operation),
                Err(e) => Err(e)
            }
        } else {
            self.operations.push(operation);
            Ok(())
        }
    }
    pub fn undo(&mut self) -> Result<Box<dyn TextBufferOperation>, TBOperationError> {
        let operation = self.operations.pop();
        if let Some(mut operation) = operation {
            let result = operation.undo(self);
            match result {
                Ok(()) => Ok(operation),
                Err(e) => Err(e)
            }
        } else {
            Err(TBOperationError::LogicError(Some("nothing to undo".into())))
        }
    }

    fn resolve_operation_error(&mut self, error: TBOperationError) -> Result<(), TBOperationError> {
        match error {
            TBOperationError::GapTooSmall { required } => {
                let arg = if required > Self::DEFAULT_GAP_SIZE {
                    required
                } else {
                    Self::DEFAULT_GAP_SIZE
                };
                self.realloc_gap(arg);
                Ok(())
            },
            error => Err(error)
        }
    }



    pub fn get_cursor(&self) -> usize {
        self.cursor
    }
    pub fn get_cursor_mut(&mut self) -> &mut usize {
        &mut self.cursor
    }

    pub fn get_gap_end(&self) -> usize {
        self.gap_end
    }
    pub fn get_gap_end_mut(&mut self) -> &mut usize {
        &mut self.gap_end
    }

    pub fn get_content(&self) -> &Vec<fixed_char> {
        &self.content
    }
    pub fn get_content_mut(&mut self) -> &mut Vec<fixed_char> {
        &mut self.content
    }

    pub fn get_length(&self) -> usize {
        self.metrics.length
    }
    pub fn get_length_raw(&self) -> usize {
        self.content.len()
    }
    pub fn get_length_mut(&mut self) -> &mut usize {
        &mut self.metrics.length
    }

    pub fn get_linebreak_iter(&self) -> Iter<usize> {

        self.metrics.get_new_line_order().iter()
    }
    pub fn get_linebreak(&self, order: usize) -> Option<usize> {
        if order == self.metrics.get_new_line_order().len() {
            return Some(self.get_length_raw());
        }
        if let Some(i) = self.metrics.get_new_line_order().get(order) {
            Some(*i)
        } else {
            None
        }
    }
    pub fn set_linebreak(&mut self, gap_index: usize) {
        self.metrics.set_linebreak(gap_index);
    }
    pub fn remove_linebreak(&mut self, gap_index: usize) {
        self.metrics.remove_linebreak(gap_index);
    }




    pub fn string_raw(&self) -> String {
        self.content.iter().map(|c| *c as char).collect()
    }
    pub fn string(&self) -> String {
        self.content.iter().enumerate().filter_map(|(i, c)| {
            if i < self.cursor || i >= self.gap_end {
                Some(*c as char)
            } else {
                None
            }
        }).collect()
    }

    fn realloc_gap(&mut self, size: usize) {
        if self.gap_end-self.cursor >= size { return; } // don't un realloc?
        let diff = size - (self.gap_end-self.cursor);
        self.content.splice(self.cursor..self.cursor, vec![' ' as fixed_char; diff]);

        for lb in self.metrics.new_lines_order.iter_mut() {
            if *lb >= self.gap_end {
                *lb += diff;
            }
        }

        self.gap_end += diff;
    }
}



#[cfg(test)]
mod test {
    use crate::textedit::operation::{CursorLeft, CursorRight, DeleteBack, InsertChar, InsertLinebreak, InsertString};
    // use crate::textedit::traverse_ops::current_line;
    use super::*;

    #[test]
    fn insert_char() {
        let mut buf = TextBuffer::new();

        buf.apply(Box::new(InsertChar('1')));

        assert_eq!(buf.cursor, 1);
        assert_eq!(buf.metrics.length, 1);
        assert_eq!("1".to_string(), buf.string());


        buf.apply(Box::new(InsertChar('2')));
        buf.apply(Box::new(InsertChar('3')));
        buf.apply(Box::new(InsertChar('4')));

        assert_eq!(buf.cursor, 4);
        assert_eq!(buf.metrics.length, 4);
        assert_eq!("1234".to_string(), buf.string());
    }

    #[test]
    fn insert_string() {
        let mut buf = TextBuffer::new();

        let string1 = "foobar".to_string();

        buf.apply(Box::new(InsertString::new(string1.clone())));

        assert_eq!(buf.string(), string1);
    }

    #[test]
    fn cursor_movements() {
        let mut buf = TextBuffer::new();

        buf.apply(Box::new(InsertChar('0')));
        buf.apply(Box::new(InsertChar('1')));
        buf.apply(Box::new(InsertChar('2')));

        buf.apply(Box::new(CursorLeft(1)));
        assert_eq!(buf.cursor, 2);

        buf.apply(Box::new(InsertChar('N')));
        assert_eq!("01N2".to_string(), buf.string());


        buf.apply(Box::new(CursorLeft(3)));
        assert_eq!(buf.cursor, 0);

        buf.apply(Box::new(InsertChar('A')));
        buf.apply(Box::new(InsertChar('B')));
        assert_eq!("AB01N2".to_string(), buf.string());


        buf.apply(Box::new(CursorRight(2)));
        assert_eq!(buf.cursor, 4);
        buf.apply(Box::new(InsertChar(' ')));
        assert_eq!("AB01 N2".to_string(), buf.string());
    }


    #[test]
    fn gap_reallocates_correctly() {
        let mut buf = TextBuffer::new();
        buf.apply(Box::new(InsertChar('0')));
        buf.apply(Box::new(InsertChar('1')));
        buf.apply(Box::new(InsertChar('2')));
        buf.apply(Box::new(InsertChar('3')));
        buf.apply(Box::new(CursorLeft(2)));

        buf.realloc_gap(2);

       assert_eq!("0123".to_string(), buf.string());
        // assert_eq!("01  23".to_string(), buf.string_raw());

        buf.apply(Box::new(InsertChar('X')));
        assert_eq!("01X23".to_string(), buf.string());
    }
    #[test]
    fn apply_operation_undo() {
        let mut buf = TextBuffer::new();

        buf.apply(Box::new(InsertChar('0')));
        buf.apply(Box::new(InsertChar('1')));
        buf.apply(Box::new(InsertChar('2')));

        assert_eq!("012".to_string(), buf.string());

        buf.undo();
        assert_eq!("01".to_string(), buf.string());

        buf.undo();
        assert_eq!("0".to_string(), buf.string());
    }

    #[test]
    fn cursor_movements_undo() {
        let mut buf = TextBuffer::new();

        buf.apply(Box::new(InsertChar('0')));
        buf.apply(Box::new(InsertChar('1')));
        buf.apply(Box::new(InsertChar('2')));


        buf.apply(Box::new(CursorLeft(1)));
        assert_eq!(buf.cursor, 2);
        buf.undo();
        assert_eq!(buf.cursor, 3);


        buf.apply(Box::new(CursorLeft(2)));
        buf.apply(Box::new(CursorRight(1)));
        assert_eq!(buf.cursor, 2);
        buf.undo();
        assert_eq!(buf.cursor, 1);
    }

    #[test]
    fn insert_char_reallocs() {
        let mut buf = TextBuffer::new();

        buf.apply(Box::new(InsertChar('0')));
        buf.apply(Box::new(InsertChar('1')));

        buf.apply(Box::new(CursorLeft(1)));

        // buf.realloc_gap(0);
        // assert_eq!(buf.gap_end-buf.cursor, 0);

        buf.apply(Box::new(InsertChar('2')));
        assert!(buf.gap_end-buf.cursor>2); // meaning it fully realloced
        assert_eq!("021".to_string(), buf.string());
    }
    #[test]
    fn insert_string_reallocs() {
        let mut buf = TextBuffer::new();

        let string1 = "foobar".to_string();

        buf.realloc_gap(1);
        assert_eq!(buf.gap_end-buf.cursor, 1);

        buf.apply(Box::new(InsertString::new(string1.clone())));
        assert!(buf.gap_end-buf.cursor>2); // meaning it fully realloced
        assert_eq!(buf.string(), string1);
    }



    #[test]
    fn get_line() {
        let mut buf = TextBuffer::new();
        buf.apply(Box::new(InsertChar('0')));
        buf.apply(Box::new(InsertChar('1')));
        buf.apply(Box::new(InsertChar('2')));
        buf.apply(Box::new(InsertLinebreak));
        buf.apply(Box::new(InsertChar('3')));

        // assert_eq!(current_line(buf.cursor, &buf.metrics),1);

        buf.apply(Box::new(CursorLeft(1)));
        // assert_eq!(current_line(buf.cursor, &buf.metrics),1);

        buf.apply(Box::new(CursorLeft(1)));
        // assert_eq!(current_line(buf.cursor, &buf.metrics),0);
    }









    #[test]
    fn linebreak_basic() {
        let mut buf = TextBuffer::new();
        buf.apply(Box::new(InsertLinebreak));

        assert_eq!(*buf.get_linebreak_iter().next().unwrap(), 0);

        buf.apply(Box::new(InsertLinebreak));
        assert_eq!(*buf.get_linebreak_iter().next().unwrap(), 1);
    }

    #[test]
    fn linebreak_removed() {
        let mut buf = TextBuffer::new();
        buf.apply(Box::new(InsertLinebreak));

        assert_eq!(*buf.get_linebreak_iter().next().unwrap(), 0);

        buf.undo();
        assert_eq!(buf.get_linebreak_iter().len(), 0);

        buf.apply(Box::new(InsertLinebreak));
        assert_eq!(*buf.get_linebreak_iter().next().unwrap(), 0);

        buf.apply(Box::new(DeleteBack::new(1)));
        assert_eq!(buf.get_linebreak_iter().len(), 0);
    }


    #[test]
    fn cursor_shifts_linebreak() {
        let mut buf = TextBuffer::new();
        let string1 = "hello".to_string();
        buf.apply(Box::new(InsertString::new(string1.clone())));

        buf.apply(Box::new(InsertLinebreak));

        assert_eq!(*buf.get_linebreak_iter().next().unwrap(), 5);


        buf.apply(Box::new(CursorLeft(1)));
        assert_eq!(*buf.get_linebreak_iter().next().unwrap(), buf.get_gap_end());

        buf.apply(Box::new(CursorRight(1)));
        assert_eq!(*buf.get_linebreak_iter().next().unwrap(), 5);

        buf.apply(Box::new(CursorLeft(2)));
        assert_eq!(*buf.get_linebreak_iter().next().unwrap(), buf.get_gap_end()+1);
    }
}