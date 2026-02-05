use crate::plot::Plot;
use crate::window::Window;

pub enum PopUpPositionOption {
    Centered(isize),
    NegativeBound(isize),
    PositiveBound(isize),
}

pub struct PopUpPosition {
    pub row: PopUpPositionOption,
    pub col: PopUpPositionOption,
}

pub enum PopUpDimensionOption {
    Fixed(usize),
    Percent(f32),
}

pub struct PopUpDimension {
    pub row: PopUpDimensionOption,
    pub col: PopUpDimensionOption,
}


pub trait PopUp: Window {
    fn position(&self) -> PopUpPosition;
    fn dimension(&self) -> PopUpDimension;
    fn term_pos(&self, dim: &Plot) -> Plot {
        Plot::new(
            to_term_pos(dim.row, self.position().row),
            to_term_pos(dim.col, self.position().col)
        )
    }
    fn term_dim(&self, dim: &Plot) -> Plot {
        Plot::new(
            to_term_dim(dim.row, self.dimension().row),
            to_term_dim(dim.col, self.dimension().col)
        )
    }
    fn local(&self) -> bool;
}


fn to_term_pos(avail: usize, pos: PopUpPositionOption) -> usize {
    (match pos {
        PopUpPositionOption::Centered(offset) => offset + (avail as isize/2),
        PopUpPositionOption::NegativeBound(offset) => offset,
        PopUpPositionOption::PositiveBound(offset) => avail as isize - offset,
    }) as usize
}
fn to_term_dim(avail: usize, pos: PopUpDimensionOption) -> usize {
    match pos {
        PopUpDimensionOption::Fixed(n) => n,
        PopUpDimensionOption::Percent(p) => ((avail as f32) * p) as usize
    }
}
