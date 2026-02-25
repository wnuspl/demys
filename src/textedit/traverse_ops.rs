use crate::textedit::buffer::TBMetrics;
use crate::textedit::fixed_char;
use crate::textedit::operation::{CursorLeft, CursorRight, TBOperationError, TextBufferOperation};
use crate::textedit::operation::TBOperationError::MovesOutOfBounds;

pub fn current_line(cursor: usize, metrics: &TBMetrics) -> usize {
    let ordered = metrics.get_new_line_order();
    if ordered.len() == 0 { return 0; }
    let next_line = ordered.iter().position(|lb| *lb > cursor);
    if let Some(next_line) = next_line {
        next_line-1
    } else {
        ordered.len()-1
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
    fn apply(&mut self, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, metrics: &mut TBMetrics) -> Result<(), TBOperationError> {
        let current_line = current_line(*cursor, metrics);
        let current_line_start = metrics.get_new_line_order()[current_line];

        let target_line = if self.down {
            current_line + self.count
        } else {
            if current_line == 0 { return Err(MovesOutOfBounds); }
            current_line - self.count
        };

        if let Some(target_line_start) = metrics.get_new_line_order().get(target_line) {
            let to_target_line = if self.down {
                target_line_start-*gap_end
            } else {
                *cursor-target_line_start
            };


            let target_line_end;
            if let Some(after_line_start) = metrics.get_new_line_order().get(target_line + 1) {
                target_line_end = after_line_start-1;
            } else {
                target_line_end = content.len();
            }

            let to_next_line_end = target_line_end-target_line_start;
            let to_current_col = *cursor-current_line_start; // cursor movement to get to current col
            let col_move = std::cmp::min(to_next_line_end, to_current_col);


            let op: Box<dyn TextBufferOperation> = if self.down {
                Box::new(CursorRight(to_target_line + col_move))
            } else {
                Box::new(CursorLeft(to_target_line - col_move))
            };
            self.op = Some(op);
            self.op.as_mut().unwrap().apply(cursor, gap_end, content, metrics)
        } else {
            Err(MovesOutOfBounds)
        }
    }
    fn undo(&mut self, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, metrics: &mut TBMetrics) -> Result<(), TBOperationError> {
        if self.op.is_none() { return Err(TBOperationError::LogicError(None)); }

        self.op.as_mut().unwrap().undo(cursor, gap_end, content, metrics)
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
    fn apply(&mut self, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, metrics: &mut TBMetrics) -> Result<(), TBOperationError> {
        let current_line = current_line(*cursor, metrics);

        if let Some(next_line_start) = metrics.get_new_line_order().get(current_line + 1) {
            // move to next_line_start-1

            let subop = CursorRight((next_line_start-1)-*gap_end);
            self.0 = Some(subop);
            self.0.as_mut().unwrap().apply(cursor, gap_end, content, metrics)
        } else {
            // move to length-gap_end -1
            let subop = CursorRight(metrics.length-*cursor);
            self.0 = Some(subop);
            self.0.as_mut().unwrap().apply(cursor, gap_end, content, metrics)
        }
    }
    fn undo(&mut self, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, metrics: &mut TBMetrics) -> Result<(), TBOperationError> {
        if self.0.is_none() { return Err(TBOperationError::LogicError(None)); }
        self.0.as_mut().unwrap().undo(cursor, gap_end, content, metrics)
    }
}