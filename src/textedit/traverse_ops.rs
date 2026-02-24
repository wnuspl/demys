use crate::textedit::fixed_char;
use crate::textedit::operation::{TBOperationError, TextBufferOperation};

pub struct DownLine {
    init: Option<usize>
}

impl TextBufferOperation for DownLine {
    fn modifies(&self) -> bool {
        false
    }
    fn apply(&mut self, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, length: &mut usize) -> Result<(), TBOperationError> {
        // find next line
        self.init = Some(*cursor);
        for char in &content[*cursor..] {
            if *char == '\n' as fixed_char {
            }
        }

        Ok(())
    }
    fn undo(&mut self, cursor: &mut usize, gap_end: &mut usize, content: &mut Vec<fixed_char>, length: &mut usize) -> Result<(), TBOperationError> {
        Ok(())
    }
}