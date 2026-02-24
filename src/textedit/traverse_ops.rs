use crate::textedit::buffer::TBMetrics;
use crate::textedit::fixed_char;
use crate::textedit::operation::{CursorLeft, CursorRight, TBOperationError, TextBufferOperation};


fn undo_subop(subop: &mut Option<Box<dyn TextBufferOperation>>, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, metrics: &mut TBMetrics) -> Result<(), TBOperationError> {
    if subop.is_none() { return Err(TBOperationError::LogicError(None)); }

    subop.as_mut().unwrap().undo(cursor, gap_end, content, metrics)
}

fn find_line_break<'a>(iter: impl Iterator<Item=&'a fixed_char>) -> Option<usize> {
    for (i,char) in iter.enumerate() {
        if *char == '\n' as fixed_char {
            return Some(i+1);
        }
    }
    return None;
}

pub struct DownLine {
    count: usize,
    subop: Option<Box<dyn TextBufferOperation>>
}
impl DownLine {
    pub fn new(count: usize) -> Self { Self { count, subop: None } }
}

impl TextBufferOperation for DownLine {
    fn modifies(&self) -> bool {
        false
    }
    fn apply(&mut self, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, metrics: &mut TBMetrics) -> Result<(), TBOperationError> {
        // find next line
        let iter = content[*cursor..].iter();
        if let Some(lb) = find_line_break(iter) {
            self.subop = Some(Box::new(CursorRight(lb)));
            self.subop.as_mut().unwrap().apply(cursor, gap_end, content, metrics)
        } else {
            Err(TBOperationError::MovesOutOfBounds)
        }
    }
    fn undo(&mut self, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, metrics: &mut TBMetrics) -> Result<(), TBOperationError> {
        undo_subop(&mut self.subop, cursor, gap_end, content, metrics)
    }
}



pub struct UpLine {
    count: usize,
    subop: Option<Box<dyn TextBufferOperation>>
}
impl UpLine {
    pub fn new(count: usize) -> Self { Self { count, subop: None } }
}

impl TextBufferOperation for UpLine {
    fn modifies(&self) -> bool {
        false
    }
    fn apply(&mut self, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, metrics: &mut TBMetrics) -> Result<(), TBOperationError> {
        // find next line
        let iter = content[..*cursor].iter().rev();
        if let Some(lb) = find_line_break(iter) {
            self.subop = Some(Box::new(CursorLeft(lb)));
            self.subop.as_mut().unwrap().apply(cursor, gap_end, content, metrics)
        } else {
            Err(TBOperationError::MovesOutOfBounds)
        }
    }
    fn undo(&mut self, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, metrics: &mut TBMetrics) -> Result<(), TBOperationError> {
        undo_subop(&mut self.subop, cursor, gap_end, content, metrics)
    }
}
