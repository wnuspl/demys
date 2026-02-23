use std::cmp::min;
use std::error::Error;
use std::mem::discriminant;
use crate::plot::Plot;

pub struct WindowSpace {
    pub dim: Plot,
    pub start: Plot
}
pub struct BorderSpace {
    pub vertical: bool,
    pub start: Plot,
    pub length: usize,
    pub thickness: usize
}
pub struct Layout {
    window_space: Vec<WindowSpace>,
    border_space: Vec<BorderSpace>,
    pub(crate) grid: Grid,
    generated: bool,
    dim: Plot
}
pub struct Grid {
    vertical_major: bool,
    major_scales: Vec<f32>,
    minor_scales: Vec<Vec<f32>>
}


// all elements in output sum to 1
// functional awesome :sunglasses:
fn to_distribution_vec(vec: &Vec<f32>) -> Vec<f32> {
    let sum = vec.iter().sum::<f32>();
    vec.iter().map(|x| x / sum).collect()
}

impl Grid {
    pub fn new() -> Grid {
        Grid {
            vertical_major: false,
            major_scales: vec![1.0],
            minor_scales: vec![vec![1.0]],
        }
    }
    /// Add another window split along the major axis
    pub fn split_major(&mut self) {
        let current_splits = self.major_scales.len();

        self.major_scales.push(1.0/current_splits as f32);
        self.major_scales = to_distribution_vec(&self.major_scales);

        self.minor_scales.push(vec![1.0]);
    }
    /// Add another window split along the minor axis
    pub fn split_minor(&mut self, index: usize) -> Result<(), Box<dyn Error>> {
        if let Some(minor) = self.minor_scales.get_mut(index) {
            let current_splits = minor.len();
            minor.push(1.0/current_splits as f32);

            *minor = to_distribution_vec(minor);
            Ok(())
        } else {
            Err("".into())
        }
    }

    pub fn generate(&self, dim: Plot) -> (Vec<WindowSpace>, Vec<BorderSpace>) {
        let mut window_out = Vec::new();
        let mut border_out = Vec::new();


        let mut major_offset = 0;
        let mut major_iter = self.major_scales.iter().zip(self.minor_scales.iter()).peekable();

        let major_available = if self.vertical_major {
            dim.col
        } else {
            dim.row
        } - (self.major_scales.len()-1)*1;

        while let Some((major_scale, minor)) = major_iter.next() {
            let major_size = (major_available as f32*major_scale) as usize;
            let mut minor_iter = minor.iter().peekable();
            let mut minor_offset = 0;
            let minor_available = if self.vertical_major {
                dim.row
            } else {
                dim.col
            } - (minor.len()-1)*1;

            while let Some(minor_scale) = minor_iter.next() {
                let minor_size = (minor_available as f32*minor_scale) as usize;
                let start;
                let dim;
                if self.vertical_major {
                    start = Plot::new(minor_offset, major_offset);
                    dim =  Plot::new(minor_size, major_size);
                } else {
                    start = Plot::new(major_offset, minor_offset);
                    dim =  Plot::new(major_size, minor_size);
                };

                let window = WindowSpace {
                    dim,
                    start
                };




                if minor_iter.peek().is_some() {
                    minor_offset += minor_size;

                    // // border
                    let thickness = 1;

                    let start = if self.vertical_major {
                        Plot::new(minor_offset, major_offset)
                    } else {
                        Plot::new(major_offset, minor_offset)
                    };

                    border_out.push(BorderSpace {
                        start,
                        vertical: !self.vertical_major,
                        length: major_size,
                        thickness
                    });

                    minor_offset += thickness;
               }

                window_out.push(window);
            }

            if major_iter.peek().is_some() {
                major_offset += major_size;
                let thickness = 1;

                let start = if self.vertical_major {
                    Plot::new(0, major_offset)
                } else {
                    Plot::new(major_offset, 0)
                };

                border_out.push(BorderSpace {
                    start,
                    vertical: self.vertical_major,
                    length: if self.vertical_major { dim.row } else { dim.col },
                    thickness
                });

                major_offset += thickness;
            }
        }

        (window_out, border_out)
    }

}



impl Layout {
    pub fn new(dim: Plot) -> Layout {
        Layout {
            window_space: Vec::new(),
            border_space: Vec::new(),
            grid: Grid::new(),
            generated: false,
            dim
        }
    }
    pub fn set_dim(&mut self, dim: Plot) {
        self.dim = dim;
        self.generated = false;
        self.generate();
    }
    pub fn generate(&mut self) {
        if self.generated { return; }

        let result = self.grid.generate(self.dim);
        self.window_space = result.0;
        self.border_space = result.1;
    }
    pub fn get_windows(&self) -> &Vec<WindowSpace> {
        &self.window_space
    }
    pub fn get_borders(&self) -> &Vec<BorderSpace> {
        &self.border_space
    }
}






#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn creates_splits() {
        let mut grid = Grid::new();

        grid.split_major();
        assert_eq!(grid.major_scales.len(), 2);

        grid.split_major();
        assert_eq!(grid.major_scales.len(), 3);

        grid.split_minor(0);
        assert_eq!(grid.major_scales.len(), 3);
        assert_eq!(grid.minor_scales[0].len(), 2);

        grid.split_minor(0);
        assert_eq!(grid.minor_scales[0].len(), 3);
    }

    #[test]
    fn splits_are_even() {
        let mut grid = Grid::new();

        grid.split_major();
        grid.split_major();
        assert_eq!(grid.major_scales[0], grid.major_scales[1]);
        assert_eq!(grid.major_scales[1], grid.major_scales[2]);

        grid.split_minor(0);
        assert_eq!(grid.minor_scales[0][0], grid.minor_scales[0][1]);
    }


    #[test]
    fn window_size_single() {
        let mut grid = Grid::new();


        let res = grid.generate(Plot::new(40,100));
        let window = &res.0[0];

        assert_eq!(window.dim, Plot::new(40, 100));



        grid.split_major();
        let res2 = grid.generate(Plot::new(40,100));
        let window1 = &res2.0[0];
        let window2 = &res2.0[1];

        assert_eq!(window1.dim, Plot::new(40, 50));
        assert_eq!(window2.dim, Plot::new(40, 50));



        grid.split_minor(0);
        let res3 = grid.generate(Plot::new(40,100));

        assert_eq!(res3.0[0].dim, Plot::new(20, 50));
        assert_eq!(res3.0[1].dim, Plot::new(20, 50));

    }
}