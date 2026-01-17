use std::fmt::Display;

pub mod buffer;
pub mod window;
pub mod window_manager;


#[derive(Clone,Copy)]
pub struct GridPos {
    row: usize,
    col: usize
}

impl From<(usize,usize)> for GridPos {
    fn from(value: (usize,usize)) -> Self {
        Self {
            row: value.0,
            col: value.1
        }
    }
}

impl From<GridPos> for (usize,usize) {
    fn from(value: GridPos) -> Self {
        (value.row, value.col)
    }
}

impl Display for GridPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.row, self.col)
    }
}