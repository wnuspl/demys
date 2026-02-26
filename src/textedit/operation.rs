use std::collections::BTreeSet;
use crate::textedit::buffer::TextBuffer;
use crate::textedit::fixed_char;

pub enum TBOperationError {
    GapTooSmall {
        required: usize
    },
    MovesOutOfBounds,
    LogicError(Option<String>)
}
pub trait TextBufferOperation {
    fn modifies(&self) -> bool { true }
    fn apply(&mut self, buffer: &mut TextBuffer) -> Result<(), TBOperationError>;
    fn undo(&mut self, buffer: &mut TextBuffer) -> Result<(), TBOperationError>;
}

pub struct InsertChar(pub char);

impl TextBufferOperation for InsertChar {
    fn apply(&mut self, buffer: &mut TextBuffer) -> Result<(), TBOperationError> {
        let cursor = buffer.get_cursor();
        if buffer.get_gap_end()-cursor < 1 { return Err(TBOperationError::GapTooSmall { required: 1 }); }

        let slice = &vec![self.0 as fixed_char];
        buffer.get_content_mut()[cursor..(cursor+1)].copy_from_slice(slice);
        *buffer.get_length_mut() += 1;
        *buffer.get_cursor_mut() += 1;
        Ok(())
    }
    fn undo(&mut self, buffer: &mut TextBuffer) -> Result<(), TBOperationError> {
        if buffer.get_cursor() == 0 { return Err(TBOperationError::MovesOutOfBounds); }
        *buffer.get_length_mut() -= 1;
        *buffer.get_cursor_mut() -= 1;
        Ok(())
    }
}

pub struct InsertLinebreak;

impl TextBufferOperation for InsertLinebreak {
    fn apply(&mut self, buffer: &mut TextBuffer) -> Result<(), TBOperationError> {
        InsertChar('\n').apply(buffer)?;
        buffer.set_linebreak_at(buffer.get_cursor()-1);
        Ok(())
    }
    fn undo(&mut self, buffer: &mut TextBuffer) -> Result<(), TBOperationError> {
        InsertChar('\n').undo(buffer)?;
        buffer.remove_linebreak_at(buffer.get_cursor());
        Ok(())
    }
}



pub struct DeleteBack {
    count: usize,
    removed: Option<Vec<fixed_char>>
}
impl DeleteBack {
    pub fn new(count: usize) -> Self { Self { count, removed: None } }
}

impl TextBufferOperation for DeleteBack {
    fn apply(&mut self, buffer: &mut TextBuffer) -> Result<(), TBOperationError> {
        let n = self.count;
        let cursor = buffer.get_cursor();
        if cursor < n { return Err(TBOperationError::MovesOutOfBounds); }

        let content = buffer.get_content();
        let moved = content[(cursor-n)..cursor].to_vec();

        for (i, ch) in moved.iter().enumerate() {
            if *ch == '\n' as fixed_char {
                buffer.remove_linebreak_at(cursor-n+i);
            }
        }

        self.removed = Some(Vec::from(moved));

        *buffer.get_cursor_mut() -= n;
        *buffer.get_length_mut() -= n;

        Ok(())
    }
    fn undo(&mut self, buffer: &mut TextBuffer) -> Result<(), TBOperationError> {
        let n = self.count;
        let cursor = buffer.get_cursor();
        if buffer.get_gap_end()-cursor < n { return Err(TBOperationError::GapTooSmall { required: n }); }

        if self.removed.is_none() { return Err(TBOperationError::LogicError(Some("no string found, operation hasn't been applied".to_string())))}

        for (i, ch) in self.removed.as_ref().unwrap().iter().enumerate() {
            if *ch == '\n' as fixed_char {
                buffer.set_linebreak_at(cursor+i);
            }
        }

        buffer.get_content_mut()[cursor..(cursor+n)].copy_from_slice(self.removed.as_ref().unwrap());
        *buffer.get_length_mut() += n;
        *buffer.get_cursor_mut()+= n;

        Ok(())
    }
}
pub struct InsertString(pub Vec<fixed_char>);
impl InsertString {
    pub fn new(string: String) -> Self { Self(Vec::from(string)) }
}

impl TextBufferOperation for InsertString {
    fn apply(&mut self, buffer: &mut TextBuffer) -> Result<(), TBOperationError> {
        let n = self.0.len();
        let cursor = buffer.get_cursor();
        if buffer.get_gap_end()-cursor < n { return Err(TBOperationError::GapTooSmall { required: n }); }

        for (i, ch) in self.0.iter().enumerate() {
            if *ch == '\n' as fixed_char {
                buffer.set_linebreak_at(cursor+i);
            }
        }

        buffer.get_content_mut()[cursor..(cursor + n)].copy_from_slice(&self.0);
        *buffer.get_length_mut() += n;
        *buffer.get_cursor_mut() += n;

        Ok(())
    }
    fn undo(&mut self, buffer: &mut TextBuffer) -> Result<(), TBOperationError> {
        let n = self.0.len();
        let cursor = buffer.get_cursor();
        if cursor < n { return Err(TBOperationError::MovesOutOfBounds); }

        for (i, ch) in self.0.iter().enumerate() {
            if *ch == '\n' as fixed_char {
                buffer.remove_linebreak_at(cursor-n+i);
           }
        }

        *buffer.get_length_mut() -= n;
        *buffer.get_cursor_mut() -= n;
        Ok(())
    }
}






pub struct CursorRight(pub usize);
pub struct CursorLeft(pub usize);

fn _cursor_right(count: usize, buffer: &mut TextBuffer) -> Result<(), TBOperationError> {
    let cursor = buffer.get_cursor();
    let gap_end = buffer.get_gap_end();
    if (cursor+count) > buffer.get_length() { return Err(TBOperationError::MovesOutOfBounds); }

    let moved = &buffer.get_content_mut()[gap_end..(gap_end+count)].to_vec();
    for (i, ch) in moved.iter().enumerate() {
        if *ch == '\n' as fixed_char {
            let gap_index = gap_end+i;
            buffer.remove_linebreak_at(gap_index);
            buffer.set_linebreak_at(cursor+i);
        }
    }

    buffer.get_content_mut()[cursor..(cursor + count)].copy_from_slice(moved);

    *buffer.get_cursor_mut() += count;
    *buffer.get_gap_end_mut() += count;
    Ok(())
}
fn _cursor_left(count: usize, buffer: &mut TextBuffer) -> Result<(), TBOperationError> {
    let cursor = buffer.get_cursor();
    let gap_end = buffer.get_gap_end();
    if cursor < count { return Err(TBOperationError::MovesOutOfBounds); }

    let moved = &buffer.get_content_mut()[(cursor-count)..cursor].to_vec();
    for (i, ch) in moved.iter().enumerate() {
        if *ch == '\n' as fixed_char {
            let gap_index = (cursor-count) + i;
            buffer.remove_linebreak_at(gap_index);
            buffer.set_linebreak_at((buffer.get_gap_end()-count)+i);
        }
    }

    buffer.get_content_mut()[(gap_end-count)..gap_end].copy_from_slice(moved);

    *buffer.get_cursor_mut() -= count;
    *buffer.get_gap_end_mut() -= count;
    Ok(())
}

impl TextBufferOperation for CursorRight {
    fn modifies(&self) -> bool {
        true
    }
    fn apply(&mut self, buffer: &mut TextBuffer) -> Result<(), TBOperationError> {
       _cursor_right(self.0, buffer)
    }
    fn undo(&mut self, buffer: &mut TextBuffer) -> Result<(), TBOperationError> {
        _cursor_left(self.0, buffer)
    }
}

impl TextBufferOperation for CursorLeft {
    fn modifies(&self) -> bool {
        true
    }
    fn apply(&mut self, buffer: &mut TextBuffer) -> Result<(), TBOperationError> {
        _cursor_left(self.0, buffer)
    }
    fn undo(&mut self, buffer: &mut TextBuffer) -> Result<(), TBOperationError> {
        _cursor_right(self.0, buffer)
    }
}
