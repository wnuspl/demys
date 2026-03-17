use crate::textedit::buffer::TextBuffer;
use crate::textedit::operation::{CursorLeft, CursorRight, TBOperationError, TextBufferOperation};
use crate::textedit::operation::TBOperationError::MovesOutOfBounds;

pub fn current_line(buffer: &mut TextBuffer) -> usize {
    let mut ordered = buffer.get_linebreak_iter();
    let cursor = buffer.get_cursor();

    if ordered.len() == 0 { return 0; }
    if cursor < buffer.get_linebreak(0).unwrap() { return 0; }

    let next_line = ordered.position(|lb| *lb >= cursor);
    if let Some(next_line) = next_line {
        next_line
    } else {
        buffer.get_linebreak_iter().len()
    }
}

pub struct LineMovement {
    count: usize,
    op: Option<Box<dyn TextBufferOperation>>,
    down: bool
}
impl LineMovement {
    pub fn down(count: usize) -> Self {
        Self {
            count, op: None, down: true
        }
    }
    pub fn up(count: usize) -> Self {
        Self {
            count, op: None, down: false
        }
    }
}

impl TextBufferOperation for LineMovement {
    fn modifies(&self) -> bool {
        false
    }
    fn apply(&mut self, buffer: &mut TextBuffer) -> Result<(), TBOperationError> {
        let cursor = buffer.get_cursor();
        let current_line = current_line(buffer);

        let target_line = if self.down {
            current_line + self.count
        } else {
            if self.count > current_line { return Err(MovesOutOfBounds); }
            current_line - self.count
        };

        if let Some(target_line_end) = buffer.get_linebreak(target_line) {
            let to_target_line_end= if self.down {
                target_line_end - buffer.get_gap_end()
            } else {
                cursor - target_line_end
            };


            let op: Box<dyn TextBufferOperation> = if self.down {
                Box::new(CursorRight(to_target_line_end))
            } else {
                Box::new(CursorLeft(to_target_line_end))
            };
            self.op = Some(op);
            self.op.as_mut().unwrap().apply(buffer)
        } else {
            Err(MovesOutOfBounds)
        }
    }
    fn undo(&mut self, buffer: &mut TextBuffer) -> Result<(), TBOperationError> {
        if self.op.is_none() { return Err(TBOperationError::LogicError(None)); }

        self.op.as_mut().unwrap().undo(buffer)
    }
}










pub struct EndOfLine(Option<CursorRight>);
impl EndOfLine {
    pub fn new() -> Self { Self(None) }
}

impl TextBufferOperation for EndOfLine {
    fn modifies(&self) -> bool {
        false
    }
    fn apply(&mut self, buffer: &mut TextBuffer) -> Result<(), TBOperationError> {
        let current_line = current_line(buffer);

        if let Some(this_linebreak) = buffer.get_linebreak(current_line) {
            // move to next_line_start-1

            let subop = CursorRight((this_linebreak)-buffer.get_gap_end());
            self.0 = Some(subop);
            self.0.as_mut().unwrap().apply(buffer)
        } else {
            Err(MovesOutOfBounds)
        }
    }
    fn undo(&mut self, buffer: &mut TextBuffer) -> Result<(), TBOperationError> {
        if self.0.is_none() { return Err(TBOperationError::LogicError(None)); }
        self.0.as_mut().unwrap().undo(buffer)
    }
}



pub struct StartOfLine(Option<CursorLeft>);
impl StartOfLine {
    pub fn new() -> Self { Self(None) }
}

impl TextBufferOperation for StartOfLine {
    fn modifies(&self) -> bool {
        false
    }
    fn apply(&mut self, buffer: &mut TextBuffer) -> Result<(), TBOperationError> {
        let current_line = current_line(buffer);

        let target = if current_line == 0 {
            0
        } else {
            buffer.get_linebreak(current_line - 1).unwrap() + 1 // +1 to next line
        };

        let subop = CursorLeft(buffer.get_cursor()-target);
        self.0 = Some(subop);
        self.0.as_mut().unwrap().apply(buffer)
    }
    fn undo(&mut self, buffer: &mut TextBuffer) -> Result<(), TBOperationError> {
        if self.0.is_none() { return Err(TBOperationError::LogicError(None)); }
        self.0.as_mut().unwrap().undo(buffer)
    }
}
