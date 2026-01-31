use std::fmt::Display;
use std::ops::{Add, Sub};

#[derive(Clone,Copy)]
#[derive(Debug)]
pub struct Plot {
    pub row: usize,
    pub col: usize
}


impl From<(usize,usize)> for Plot {
    fn from(value: (usize,usize)) -> Self {
        Self {
            row: value.0,
            col: value.1
        }
    }
}
impl From<(u16,u16)> for Plot {
    fn from(value: (u16,u16)) -> Self {
        Self {
            row: value.0 as usize,
            col: value.1 as usize
        }
    }
}
impl From<Plot> for (usize,usize) {
    fn from(value: Plot) -> Self {
        (value.row, value.col)
    }
}
impl Plot {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
    pub fn transpose(mut self) -> Self {
        let temp = self.row;
        self.row = self.col;
        self.col = temp;
        self
    }
}
impl Display for Plot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.row, self.col)
    }
}
impl Add for Plot {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            row: self.row+rhs.row,
            col: self.col+rhs.col
        }
    }
}

impl PartialEq<Self> for Plot {
    fn eq(&self, other: &Self) -> bool {
        (self.row == other.row) && (self.col == other.col)
    }
}
impl Sub for Plot {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            row: self.row-rhs.row,
            col: self.col-rhs.col
        }
    }
}
