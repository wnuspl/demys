use std::mem;
use crate::GridPos;


pub struct Layout {
    pub grid: Grid,
    window_space: Vec<WindowSpace>,
    border_space: Vec<BorderSpace>,
    current_dim: GridPos
}

#[derive(Clone)]
pub enum Grid {
    Horizontal {
        body: Vec<Box<Grid>>,
        heights: Vec<f32>
    },
    Vertical {
        body: Vec<Box<Grid>>,
        widths: Vec<f32>
    },
    Single
}

pub struct WindowSpace {
    pub dim: GridPos,
    pub start: GridPos
}

pub enum BorderSpace {
    Vertical {
        length: u16,
        thickness: u16,
        start: GridPos
    },
    Horizontal {
        length: u16,
        thickness: u16,
        start: GridPos
    }
}


// all elements in output sum to 1
fn to_dist_vec(vec: &Vec<f32>) -> Vec<f32> {
    let sum = vec.iter().sum::<f32>();
    vec.iter().map(|x| x/sum).collect()
}


impl Layout {
    pub fn new() -> Self {
        Self {
            grid: Grid::new(),
            window_space: Vec::new(),
            border_space: Vec::new(),
            current_dim: (0,0).into()
        }
    }
    // set window_pos, border_pos
    pub fn generate(&mut self, dim: GridPos) {
        let mut space = self.grid.generate_space(dim, (0,0).into());

        self.window_space = mem::take(&mut space.0);
        self.border_space = mem::take(&mut space.1);
        self.current_dim = dim;
    }

    pub fn get_windows(&self) -> &Vec<WindowSpace> {
        &self.window_space
    }
    pub fn get_borders(&self) -> &Vec<BorderSpace> {
        &self.border_space
    }
}



impl Grid {
    const BORDER_THICKNESS: u16 = 1;
    pub fn new() -> Self {
        Self::Single
    }
    fn vsplit() -> Self {
        Self::Vertical {
            body: vec![Box::new(Self::new()), Box::new(Self::new())],
            widths: vec![0.5, 0.5]
        }
    }
    fn hsplit() -> Self {
        Self::Horizontal {
            body: vec![Box::new(Self::new()), Box::new(Self::new())],
            heights: vec![0.5, 0.5]
        }
    }


    // adds another split to layout, if single, defaults to vertical split
    // new window is 1/n size where n is new number of splits
    pub fn split(&mut self, vertical:bool) {
        match self {
            Self::Single => *self = if vertical { Self::vsplit() } else { Self::hsplit() },
            Self::Vertical { body, widths } => {
                if vertical {
                    // set width to 1/n
                    let w = 1.0 / (body.len() as f32);
                    widths.push(w);
                    *widths = to_dist_vec(widths);

                    body.push(Box::new(Self::Single));
                } else {
                    // self becomes inner layout, create 2 way hsplit
                    let inner = mem::replace(self, Self::hsplit());
                    *self.get_body_mut(0).unwrap() = Box::new(inner);
                }
            },
            Self::Horizontal { body, heights } => {
                if !vertical {
                    // set height to 1/n
                    let h = 1.0 / (body.len() as f32);
                    heights.push(h);
                    *heights = to_dist_vec(heights);

                    body.push(Box::new(Self::Single));
                } else {
                    // self becomes inner layout, create 2 way vsplit
                    let inner = mem::replace(self, Self::vsplit());
                    *self.get_body_mut(0).unwrap() = Box::new(inner);
                }
            },
            _ => {}
        }
    }

    pub fn get_body_mut(&mut self, idx: usize) -> Option<&mut Box<Self>> {
        match self {
            Self::Single => { None },
            Self::Vertical { body, .. }
            | Self::Horizontal { body, .. } => {
                body.get_mut(idx)
            },
            _ => None
        }
    }


    // main function of window layout
    // maps window to physical position in terminal based on size
    // return is Vec<dim, start>
    pub fn generate_space(&self, dim: GridPos, start: GridPos) -> (Vec<WindowSpace>, Vec<BorderSpace>) {
        let mut windows = Vec::new();
        let mut borders = Vec::new();
        match &self {
            Self::Single => {
                windows.push(WindowSpace {dim, start});
            }

            // Fixed width, variable height
            Self::Horizontal { body, heights } => {
                let available_height = dim.row-(body.len() as u16-1)*Self::BORDER_THICKNESS;
                let mut vertical_offset = 0;

                let mut iter = body.iter().zip(heights.iter()).peekable();

                while let Some((layout, height_percent)) = iter.next() {
                    let h = (height_percent*available_height as f32) as u16;

                    let mut inner = layout.generate_space(
                        (h, dim.col).into(),                                // size
                        (start.row+vertical_offset, start.col).into()       // start
                    );

                    windows.append(&mut inner.0);
                    borders.append(&mut inner.1);

                    vertical_offset += h;

                    if iter.peek().is_some() {
                        borders.push(BorderSpace::Horizontal {
                            thickness: Self::BORDER_THICKNESS,
                            length: dim.col,
                            start: (start.row+vertical_offset, start.col).into()
                        });

                        vertical_offset += Self::BORDER_THICKNESS;
                    }
                }
            },




            // Fixed height, variable width
            Self::Vertical { body, widths } => {
                let available_width = dim.col-(body.len() as u16-1)*Self::BORDER_THICKNESS;
                let mut horizontal_offset = 0;

                let mut iter = body.iter().zip(widths.iter()).peekable();

                while let Some((layout, width_percent)) = iter.next() {
                    let w = (width_percent*available_width as f32) as u16;


                    let mut inner = layout.generate_space(
                        (dim.row, w).into(),                                // size
                        (start.row, start.col+horizontal_offset).into()     // start
                    );

                    windows.append(&mut inner.0);
                    borders.append(&mut inner.1);

                    horizontal_offset += w;


                    if iter.peek().is_some() {
                        borders.push(BorderSpace::Vertical {
                            thickness: Self::BORDER_THICKNESS,
                            length: dim.row,
                            start: (start.row, start.col+horizontal_offset).into()
                        });
                        horizontal_offset += Self::BORDER_THICKNESS;
                    }
                }
            },
            _ => {}
        }
        (windows, borders)
    }
}
