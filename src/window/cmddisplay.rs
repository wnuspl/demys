use crate::popup::{PopUp, PopUpDimension, PopUpDimensionOption, PopUpPosition, PopUpPositionOption};
use crate::style::Canvas;
use crate::window::{Window, WindowRequest};

#[derive(Default)]
pub struct CmdDisplay {
    pub cmd: String,
    requests: Vec<WindowRequest>
}


impl Window for CmdDisplay {
    fn requests(&mut self) -> &mut Vec<WindowRequest> {
        &mut self.requests
    }

    fn draw(&self, canvas: &mut Canvas) {
        canvas.write(&format!(":{}", self.cmd).into())
    }
}

impl PopUp for CmdDisplay {
    fn dimension(&self) -> PopUpDimension {
        PopUpDimension {
            row: PopUpDimensionOption::Fixed(1),
            col: PopUpDimensionOption::Percent(1.0)
        }
    }
    fn position(&self) -> PopUpPosition {
        PopUpPosition {
            row: PopUpPositionOption::PositiveBound(1),
            col: PopUpPositionOption::Centered(0)
        }
    }
    fn local(&self) -> bool {
        false
    }
}