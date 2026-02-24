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
    fn apply(&mut self, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, length: &mut usize) -> Result<(), TBOperationError>;
    fn undo(&mut self, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, length: &mut usize) -> Result<(), TBOperationError>;
}

pub struct InsertChar(pub char);

impl TextBufferOperation for InsertChar {
    fn apply(&mut self, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, length: &mut usize) -> Result<(), TBOperationError> {
        if *gap_end-*cursor < 1 { return Err(TBOperationError::GapTooSmall { required: 1 }); }
        let slice = &vec![self.0 as fixed_char];
        content[*cursor..(*cursor+1)].copy_from_slice(slice);
        *length += 1;
        *cursor += 1;
        Ok(())
    }
    fn undo(&mut self, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, length: &mut usize) -> Result<(), TBOperationError> {
        if *cursor == 0 { return Err(TBOperationError::MovesOutOfBounds); }
        *length -= 1;
        *cursor -= 1;
        Ok(())
    }
}

pub struct DeleteBack(pub usize, Option<Vec<fixed_char>>);
impl DeleteBack {
    pub fn new(count: usize) -> Self { Self(count, None) }
}

impl TextBufferOperation for DeleteBack {
    fn apply(&mut self, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, length: &mut usize) -> Result<(), TBOperationError> {
        let n = self.0;
        if *cursor < n { return Err(TBOperationError::MovesOutOfBounds); }

        let moved = &content[(*cursor-n)..*cursor];
        self.1 = Some(Vec::from(moved.clone()));

        *length -= n;
        *cursor -= n;

        Ok(())
    }
    fn undo(&mut self, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, length: &mut usize) -> Result<(), TBOperationError> {
        let n = self.0;
        if *gap_end-*cursor < n { return Err(TBOperationError::GapTooSmall { required: n }); }

        if self.1.is_none() { return Err(TBOperationError::LogicError(Some("no string found, operation hasn't been applied".to_string())))}

        content[*cursor..(*cursor+n)].copy_from_slice( self.1.as_ref().unwrap());
        *length += n;
        *cursor += n;

        Ok(())
    }
}
pub struct InsertString(pub Vec<fixed_char>);
impl InsertString {
    pub fn new(string: String) -> Self { Self(Vec::from(string)) }
}

impl TextBufferOperation for InsertString {
    fn apply(&mut self, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, length: &mut usize) -> Result<(), TBOperationError> {
        let n = self.0.len();
        if *gap_end-*cursor < n { return Err(TBOperationError::GapTooSmall { required: n }); }

        content[*cursor..(*cursor + self.0.len())].copy_from_slice(&self.0);
        *length += n;
        *cursor += n;

        Ok(())
    }
    fn undo(&mut self, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, length: &mut usize) -> Result<(), TBOperationError> {
        let n = self.0.len();
        if *cursor < n { return Err(TBOperationError::MovesOutOfBounds); }

        *length -= n;
        *cursor -= n;
        Ok(())
    }
}



pub struct CursorRight(pub usize);
pub struct CursorLeft(pub usize);

fn _cursor_right(count: usize, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, length: &mut usize) -> Result<(), TBOperationError> {
    if (*cursor+count) > *length { return Err(TBOperationError::MovesOutOfBounds); }

    let moved = &content[*gap_end..(*gap_end+count)].to_vec();

    content[*cursor..(*cursor + count)].copy_from_slice(&moved);

    *cursor += count;
    *gap_end += count;
    Ok(())
}
fn _cursor_left(count: usize, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, length: &mut usize) -> Result<(), TBOperationError> {
    if *cursor < count { return Err(TBOperationError::MovesOutOfBounds); }

    let moved = &content[(*cursor-count)..*cursor].to_vec();

    content[(*gap_end-count)..*gap_end].copy_from_slice(&moved);

    *gap_end -= count;
    *cursor -= count;
    Ok(())
}

impl TextBufferOperation for CursorRight {
    fn modifies(&self) -> bool {
        true
    }
    fn apply(&mut self, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, length: &mut usize) -> Result<(), TBOperationError> {
       _cursor_right(self.0, cursor, gap_end, content, length)
    }
    fn undo(&mut self, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, length: &mut usize) -> Result<(), TBOperationError> {
        _cursor_left(self.0, cursor, gap_end, content, length)
    }
}

impl TextBufferOperation for CursorLeft {
    fn modifies(&self) -> bool {
        true
    }
    fn apply(&mut self, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, length: &mut usize) -> Result<(), TBOperationError> {
        _cursor_left(self.0, cursor, gap_end, content, length)
    }
    fn undo(&mut self, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, length: &mut usize) -> Result<(), TBOperationError> {
        _cursor_right(self.0, cursor, gap_end, content, length)
    }
}
