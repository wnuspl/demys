use std::io::Stdout;
use crossterm::cursor::MoveTo;
use crossterm::{queue, QueueableCommand};
use crossterm::style::{Print, SetForegroundColor, Color, Attribute, SetAttribute, ResetColor};
use crate::GridPos;

pub struct Style {
    colors: Vec<Color>,
}

pub enum StyleItem {
    Text(String),
    Color(Option<usize>),
    Bold(bool),
    LineBreak,
}

impl Style {
    pub fn new() -> Style {
        Self {
            colors: vec![Color::Black, Color::Magenta],
        }
    }


    pub fn reset(&self, stdout: &mut Stdout) {
        let _ = queue!(
            stdout,
            ResetColor,
            SetAttribute(Attribute::Reset)
        );
    }

    pub fn queue(&self, stdout: &mut Stdout, string: Vec<StyleItem>, start: GridPos, dim: GridPos) {
        let mut line = 0;
        let _ = stdout.queue(MoveTo(start.col, start.row));
        for item in string.iter() {
            match item {
                StyleItem::Text(s) => {
                    let s = s.chars().take(dim.col as usize).collect::<String>();
                    let _ = stdout.queue(Print(s));
                },
                StyleItem::Color(idx) => {
                    let i;
                    if idx.is_some() { i = idx.unwrap(); } else { i = 0; }
                    let _ = stdout.queue(SetForegroundColor(self.colors[i]));
                },
                StyleItem::Bold(bold) => {
                    let bold = if *bold { Attribute::Bold } else { Attribute::NoBold };
                    let _ = stdout.queue(SetAttribute(bold));
                },
                StyleItem::LineBreak => {
                    line += 1;
                    let _ = stdout.queue(MoveTo(start.col, start.row+line));
                }
            }
            if line > dim.row { break; }
        }
    }
}