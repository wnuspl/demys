use crate::textedit::buffer::TBMetrics;
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
    fn apply(&mut self, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, metrics: &mut TBMetrics) -> Result<(), TBOperationError>;
    fn undo(&mut self, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, metrics: &mut TBMetrics) -> Result<(), TBOperationError>;
}

pub struct InsertChar(pub char);

impl TextBufferOperation for InsertChar {
    fn apply(&mut self, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, metrics: &mut TBMetrics) -> Result<(), TBOperationError> {
        if *gap_end-*cursor < 1 { return Err(TBOperationError::GapTooSmall { required: 1 }); }
        let slice = &vec![self.0 as fixed_char];
        content[*cursor..(*cursor+1)].copy_from_slice(slice);
        metrics.length += 1;
        *cursor += 1;
        Ok(())
    }
    fn undo(&mut self, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, metrics: &mut TBMetrics) -> Result<(), TBOperationError> {
        if *cursor == 0 { return Err(TBOperationError::MovesOutOfBounds); }
        metrics.length -= 1;
        *cursor -= 1;
        Ok(())
    }
}

pub struct InsertLinebreak;

impl TextBufferOperation for InsertLinebreak {
    fn apply(&mut self, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, metrics: &mut TBMetrics) -> Result<(), TBOperationError> {
        metrics.set_linebreak_raw(*cursor);
        InsertChar('\n').apply(cursor, gap_end, content, metrics)
    }
    fn undo(&mut self, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, metrics: &mut TBMetrics) -> Result<(), TBOperationError> {
        let res = InsertChar('\n').undo(cursor, gap_end, content, metrics);
        metrics.remove_linebreak_raw(*cursor);
        res
    }
}



pub struct DeleteBack(pub usize, Option<Vec<fixed_char>>);
impl DeleteBack {
    pub fn new(count: usize) -> Self { Self(count, None) }
}

impl TextBufferOperation for DeleteBack {
    fn apply(&mut self, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, metrics: &mut TBMetrics) -> Result<(), TBOperationError> {
        let n = self.0;
        if *cursor < n { return Err(TBOperationError::MovesOutOfBounds); }

        let moved = &content[(*cursor-n)..*cursor];
        for (i, ch) in moved.iter().enumerate() {
            if *ch == '\n' as fixed_char {
                metrics.remove_linebreak_raw(*cursor-n+i);
            }
        }
        self.1 = Some(Vec::from(moved.clone()));

        metrics.length -= n;
        *cursor -= n;

        Ok(())
    }
    fn undo(&mut self, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, metrics: &mut TBMetrics) -> Result<(), TBOperationError> {
        let n = self.0;
        if *gap_end-*cursor < n { return Err(TBOperationError::GapTooSmall { required: n }); }

        if self.1.is_none() { return Err(TBOperationError::LogicError(Some("no string found, operation hasn't been applied".to_string())))}

        for (i, ch) in self.1.as_ref().unwrap().iter().enumerate() {
            if *ch == '\n' as fixed_char {
                metrics.set_linebreak_raw(*cursor+i);
            }
        }

        content[*cursor..(*cursor+n)].copy_from_slice( self.1.as_ref().unwrap());
        metrics.length += n;
        *cursor += n;

        Ok(())
    }
}
pub struct InsertString(pub Vec<fixed_char>);
impl InsertString {
    pub fn new(string: String) -> Self { Self(Vec::from(string)) }
}

impl TextBufferOperation for InsertString {
    fn apply(&mut self, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, metrics: &mut TBMetrics) -> Result<(), TBOperationError> {
        let n = self.0.len();
        if *gap_end-*cursor < n { return Err(TBOperationError::GapTooSmall { required: n }); }

        for (i, ch) in self.0.iter().enumerate() {
            if *ch == '\n' as fixed_char {
                metrics.set_linebreak_raw(*cursor+i);
            }
        }

        content[*cursor..(*cursor + self.0.len())].copy_from_slice(&self.0);
        metrics.length += n;
        *cursor += n;

        Ok(())
    }
    fn undo(&mut self, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, metrics: &mut TBMetrics) -> Result<(), TBOperationError> {
        let n = self.0.len();
        if *cursor < n { return Err(TBOperationError::MovesOutOfBounds); }

        for (i, ch) in self.0.iter().enumerate() {
            if *ch == '\n' as fixed_char {
                metrics.remove_linebreak_raw(*cursor-n+i);
            }
        }

        metrics.length -= n;
        *cursor -= n;
        Ok(())
    }
}








pub struct CursorRight(pub usize);
pub struct CursorLeft(pub usize);

fn _cursor_right(count: usize, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, metrics: &mut TBMetrics) -> Result<(), TBOperationError> {
    if (*cursor+count) > metrics.length { return Err(TBOperationError::MovesOutOfBounds); }

    let moved = &content[*gap_end..(*gap_end+count)].to_vec();
    for (i, ch) in moved.iter().enumerate() {
        if *ch == '\n' as fixed_char {
            let gap_index = *gap_end+i;
            metrics.remove_linebreak_raw(gap_index);
            metrics.set_linebreak_raw(*cursor+i);
        }
    }

    content[*cursor..(*cursor + count)].copy_from_slice(&moved);

    *cursor += count;
    *gap_end += count;
    Ok(())
}
fn _cursor_left(count: usize, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, metrics: &mut TBMetrics) -> Result<(), TBOperationError> {
    if *cursor < count { return Err(TBOperationError::MovesOutOfBounds); }

    let moved = &content[(*cursor-count)..*cursor].to_vec();
    for (i, ch) in moved.iter().enumerate() {
        if *ch == '\n' as fixed_char {
            let gap_index = (*cursor-count) + i;
            metrics.remove_linebreak_raw(gap_index);
            metrics.set_linebreak_raw((*gap_end-count)+i);
        }
    }

    content[(*gap_end-count)..*gap_end].copy_from_slice(&moved);

    *gap_end -= count;
    *cursor -= count;
    Ok(())
}

impl TextBufferOperation for CursorRight {
    fn modifies(&self) -> bool {
        true
    }
    fn apply(&mut self, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, metrics: &mut TBMetrics) -> Result<(), TBOperationError> {
       _cursor_right(self.0, cursor, gap_end, content, metrics)
    }
    fn undo(&mut self, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, metrics: &mut TBMetrics) -> Result<(), TBOperationError> {
        _cursor_left(self.0, cursor, gap_end, content, metrics)
    }
}

impl TextBufferOperation for CursorLeft {
    fn modifies(&self) -> bool {
        true
    }
    fn apply(&mut self, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, metrics: &mut TBMetrics) -> Result<(), TBOperationError> {
        _cursor_left(self.0, cursor, gap_end, content, metrics)
    }
    fn undo(&mut self, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, metrics: &mut TBMetrics) -> Result<(), TBOperationError> {
        _cursor_right(self.0, cursor, gap_end, content, metrics)
    }
}
