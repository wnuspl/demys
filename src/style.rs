use std::io::Stdout;
use crossterm::cursor::MoveTo;
use crossterm::QueueableCommand;
use crossterm::style::{Print, SetForegroundColor, Color, Attribute, SetAttribute};
use crate::GridPos;

pub struct Style {
    colors: Vec<Color>,
    border_color: usize
}

pub enum StyleItem {
    Text(String),
    Format(Option<usize>, Option<bool>, Option<bool>),
    LineBreak,
}

 pub struct StyledString(pub Vec<StyleItem>);


impl Style {
    pub fn new() -> Style {
        Self {
            colors: vec![],
            border_color: 1,
        }
    }


    pub fn queue(&self, stdout: &mut Stdout, string: StyledString, start: GridPos, dim: GridPos) {
        let mut line = 0;
        for item in string.0.iter() {
            match item {
                StyleItem::Text(s) => {
                    let s = s.chars().take(dim.col as usize).collect::<String>();
                    let _ = stdout.queue(Print(s));
                },
                StyleItem::Format(color, bold, italic) => {
                    if let Some(color) = color {
                        let _ = stdout.queue(SetForegroundColor(self.colors[*color]));
                    }
                    if let Some(bold) = bold {
                        let bold = if *bold { Attribute::Bold } else { Attribute::NoBold };
                        let _ = stdout.queue(SetAttribute(bold));
                    }
                }
                StyleItem::LineBreak => {
                    let _ = stdout.queue(MoveTo(start.col, start.row+line));
                    line += 1;
                }
            }
            if line > dim.row { break; }
        }
    }
}