use std::fmt::Display;
use std::ops::Add;
use std::error::Error;

pub mod buffer;
pub mod window;
pub mod texttab;
pub mod fstab;
pub mod window_manager;
pub mod style;
pub mod layout;

#[derive(Clone,Copy)]
pub struct GridPos {
    row: u16,
    col: u16
}


impl From<(u16,u16)> for GridPos {
    fn from(value: (u16,u16)) -> Self {
        Self {
            row: value.0,
            col: value.1
        }
    }
}
impl From<GridPos> for (u16,u16) {
    fn from(value: GridPos) -> Self {
        (value.row, value.col)
    }
}
impl GridPos {
    pub fn transpose(mut self) -> Self {
        let temp = self.row;
        self.row = self.col;
        self.col = temp;
        self
    }
}
impl Display for GridPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.row, self.col)
    }
}
impl Add for GridPos {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            row: self.row+rhs.row,
            col: self.col+rhs.col
        }
    }
}