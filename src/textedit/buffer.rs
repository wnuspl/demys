use std::error::Error;
use std::path::PathBuf;
use crate::plot::Plot;
use crate::textedit::fixed_char;
use crate::textedit::operation::{TextBufferOperation, TBOperationError};


pub struct TextBuffer {
    length: usize,
    content: Vec<fixed_char>,
    cursor: usize,
    gap_end: usize,
    operations: Vec<Box<dyn TextBufferOperation>>
}


impl From<PathBuf> for TextBuffer {
    fn from(path: PathBuf) -> Self {
        TextBuffer::new()
    }
}

impl TextBuffer {
    const DEFAULT_GAP_SIZE: usize = 100;
    pub fn new() -> TextBuffer {
        TextBuffer {
            length: 0,
            content: vec![' ' as fixed_char; TextBuffer::DEFAULT_GAP_SIZE],
            cursor: 0,
            gap_end: Self::DEFAULT_GAP_SIZE,
            operations: Vec::new()
        }
    }
    pub(crate) fn apply_operation(&mut self, mut operation: Box<dyn TextBufferOperation>) {
        let result = operation.apply(
            &mut self.cursor,
            &mut self.gap_end,
            &mut self.content,
            &mut self.length
        );

        if let Err(error) = result {
            if let Ok(()) = self.handle_operation_error(error) {
                self.apply_operation(operation);
            }
        } else {
            self.operations.push(operation);
        }
    }
    pub(crate) fn undo_operation(&mut self) {
        let operation = self.operations.pop();
        if let Some(mut operation) = operation {
            let result = operation.undo(
                &mut self.cursor,
                &mut self.gap_end,
                &mut self.content,
                &mut self.length
            );

            // handle error?
            // this should never happen
        }
    }

    fn handle_operation_error(&mut self, error: TBOperationError) -> Result<(), Box<dyn Error>> {
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
            TBOperationError::MovesOutOfBounds => Err("cannot recover".into()),
            TBOperationError::LogicError(message) => {
                if let Some(message) = message {
                    panic!("{}", message);
                } else {
                    panic!();
                }
            }
        }
    }

    fn get_line(&self) -> usize {
        let mut line = 0;
        for c in self.content.iter().take(self.cursor) {
            if *c == '\n' as fixed_char  { line += 1; }
        }
        return line;
    }

    pub fn get_cursor(&self) -> usize {
        self.cursor
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
        self.gap_end += diff;
    }
}



#[cfg(test)]
mod test {
    use crate::textedit::operation::{CursorLeft, CursorRight, InsertChar, InsertString};
    use super::*;

    #[test]
    fn insert_char() {
        let mut buf = TextBuffer::new();

        buf.apply_operation(Box::new(InsertChar('1')));

        assert_eq!(buf.cursor, 1);
        assert_eq!(buf.length, 1);
        assert_eq!("1".to_string(), buf.string());


        buf.apply_operation(Box::new(InsertChar('2')));
        buf.apply_operation(Box::new(InsertChar('3')));
        buf.apply_operation(Box::new(InsertChar('4')));

        assert_eq!(buf.cursor, 4);
        assert_eq!(buf.length, 4);
        assert_eq!("1234".to_string(), buf.string());
    }

    #[test]
    fn insert_string() {
        let mut buf = TextBuffer::new();

        let string1 = "foobar".to_string();

        buf.apply_operation(Box::new(InsertString::new(string1.clone())));

        assert_eq!(buf.string(), string1);
    }

    #[test]
    fn cursor_movements() {
        let mut buf = TextBuffer::new();

        buf.apply_operation(Box::new(InsertChar('0')));
        buf.apply_operation(Box::new(InsertChar('1')));
        buf.apply_operation(Box::new(InsertChar('2')));

        buf.apply_operation(Box::new(CursorLeft(1)));
        assert_eq!(buf.cursor, 2);

        buf.apply_operation(Box::new(InsertChar('N')));
        assert_eq!("01N2".to_string(), buf.string());


        buf.apply_operation(Box::new(CursorLeft(3)));
        assert_eq!(buf.cursor, 0);

        buf.apply_operation(Box::new(InsertChar('A')));
        buf.apply_operation(Box::new(InsertChar('B')));
        assert_eq!("AB01N2".to_string(), buf.string());


        buf.apply_operation(Box::new(CursorRight(2)));
        assert_eq!(buf.cursor, 4);
        buf.apply_operation(Box::new(InsertChar(' ')));
        assert_eq!("AB01 N2".to_string(), buf.string());
    }

    #[test]
    fn get_line() {
        let mut buf = TextBuffer::new();
        buf.apply_operation(Box::new(InsertChar('0')));
        buf.apply_operation(Box::new(InsertChar('1')));
        buf.apply_operation(Box::new(InsertChar('\n')));
        println!("{}", buf.cursor);
        for (i,c) in buf.string().chars().enumerate() {
            println!("{}, {}", c as usize, i);
        }
        assert_eq!(buf.get_line(), 1);

        buf.apply_operation(Box::new(CursorLeft(1)));
        assert_eq!(buf.get_line(), 0);

        buf.apply_operation(Box::new(InsertChar('\n')));
        assert_eq!(buf.get_line(), 1);

        buf.apply_operation(Box::new(CursorRight(1)));
        assert_eq!(buf.get_line(), 2);


    }


    #[test]
    fn gap_reallocates_correctly() {
        let mut buf = TextBuffer::new();
        buf.apply_operation(Box::new(InsertChar('0')));
        buf.apply_operation(Box::new(InsertChar('1')));
        buf.apply_operation(Box::new(InsertChar('2')));
        buf.apply_operation(Box::new(InsertChar('3')));
        buf.apply_operation(Box::new(CursorLeft(2)));

        buf.realloc_gap(2);

       assert_eq!("0123".to_string(), buf.string());
        // assert_eq!("01  23".to_string(), buf.string_raw());

        buf.apply_operation(Box::new(InsertChar('X')));
        assert_eq!("01X23".to_string(), buf.string());
    }
    #[test]
    fn apply_operation_undo() {
        let mut buf = TextBuffer::new();

        buf.apply_operation(Box::new(InsertChar('0')));
        buf.apply_operation(Box::new(InsertChar('1')));
        buf.apply_operation(Box::new(InsertChar('2')));

        assert_eq!("012".to_string(), buf.string());

        buf.undo_operation();
        assert_eq!("01".to_string(), buf.string());

        buf.undo_operation();
        assert_eq!("0".to_string(), buf.string());
    }

    #[test]
    fn cursor_movements_undo() {
        let mut buf = TextBuffer::new();

        buf.apply_operation(Box::new(InsertChar('0')));
        buf.apply_operation(Box::new(InsertChar('1')));
        buf.apply_operation(Box::new(InsertChar('2')));


        buf.apply_operation(Box::new(CursorLeft(1)));
        assert_eq!(buf.cursor, 2);
        buf.undo_operation();
        assert_eq!(buf.cursor, 3);


        buf.apply_operation(Box::new(CursorLeft(2)));
        buf.apply_operation(Box::new(CursorRight(1)));
        assert_eq!(buf.cursor, 2);
        buf.undo_operation();
        assert_eq!(buf.cursor, 1);
    }

    #[test]
    fn insert_char_reallocs() {
        let mut buf = TextBuffer::new();

        buf.apply_operation(Box::new(InsertChar('0')));
        buf.apply_operation(Box::new(InsertChar('1')));

        buf.apply_operation(Box::new(CursorLeft(1)));

        // buf.realloc_gap(0);
        // assert_eq!(buf.gap_end-buf.cursor, 0);

        buf.apply_operation(Box::new(InsertChar('2')));
        assert!(buf.gap_end-buf.cursor>2); // meaning it fully realloced
        assert_eq!("021".to_string(), buf.string());
    }
    #[test]
    fn insert_string_reallocs() {
        let mut buf = TextBuffer::new();

        let string1 = "foobar".to_string();

        // buf.realloc_gap(1);
        // assert_eq!(buf.gap_end-buf.cursor, 1);

        buf.apply_operation(Box::new(InsertString::new(string1.clone())));
        assert!(buf.gap_end-buf.cursor>2); // meaning it fully realloced
        assert_eq!(buf.string(), string1);
    }
}