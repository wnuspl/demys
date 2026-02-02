use std::io::Stdout;
use crossterm::QueueableCommand;
use crossterm::style::{Attribute, Color, SetAttribute, SetBackgroundColor, SetForegroundColor};

/// Basically a map to native terminal colors
/// Exists to be dynamically change color themes
#[derive(Copy,Clone)]
pub enum ThemeColor {
    Primary,
    Blue,
    Magenta,
    Yellow,
    Green,
    Gray,
    Black,
    White,
    Background,
}
impl From<ThemeColor> for crossterm::style::Color {
    fn from(color: ThemeColor) -> Self {
        match color {
            ThemeColor::Primary => crossterm::style::Color::Black,
            ThemeColor::Blue => crossterm::style::Color::Blue,
            ThemeColor::Magenta => crossterm::style::Color::Magenta,
            ThemeColor::Yellow => crossterm::style::Color::Yellow,
            ThemeColor::Green => crossterm::style::Color::Green,
            ThemeColor::Gray => crossterm::style::Color::Grey,
            ThemeColor::Black => crossterm::style::Color::Black,
            ThemeColor::White => crossterm::style::Color::White,
            ThemeColor::Background => crossterm::style::Color::Reset
        }
    }
}

/// Wrapper for style options
#[derive(Copy,Clone)]
#[repr(usize)]
pub enum StyleAttribute {
    Color(ThemeColor),
    Bold(bool),
    BgColor(ThemeColor),
}
impl From<StyleAttribute> for usize {
    fn from(attr: StyleAttribute) -> Self {
        match attr {
            StyleAttribute::Color(_) => 0,
            StyleAttribute::Bold(_) => 1,
            StyleAttribute::BgColor(_) => 2,
        }
    }
}

impl StyleAttribute {
    pub const COUNT: usize = 3;
    /// Apply attribute to stdout
    pub fn apply(&self, stdout: &mut Stdout) {
        match self {
            StyleAttribute::Color(color) => {
                let _ = stdout.queue(
                    SetForegroundColor((*color).into())
                );
            }
            StyleAttribute::Bold(bold) => {
                let _ = stdout.queue(
                    if *bold {
                        SetAttribute(Attribute::Bold)
                    } else {
                        SetAttribute(Attribute::NormalIntensity)
                    }
                );
            }
            StyleAttribute::BgColor(color) => {
                let _ = stdout.queue(
                    SetBackgroundColor((*color).into())
                );
            }
        }
    }
    /// Appy default version of variant to stdout
    pub fn reset(&self, stdout: &mut Stdout) {
        let _ = match self {
            StyleAttribute::Color(_) => {
                stdout.queue(SetForegroundColor(Color::Reset))
            }
            StyleAttribute::Bold(_) => {
                stdout.queue(SetAttribute(Attribute::NormalIntensity))
            }
            StyleAttribute::BgColor(_) => {
                stdout.queue(SetBackgroundColor(Color::Reset))
            }
        };
    }
}


/// Text paired with styling and writing options
pub struct StyledText {
    text: String,
    attribute: Vec<StyleAttribute>,
    wrap: bool
}
impl StyledText {
    /// Create new from string. Has no style
    pub fn new(text: String) -> Self {
        Self {
            text,
            attribute: Vec::new(),
            wrap: true,
        }
    }
    /// Create StyledText with attributes in line
    pub fn with(mut self, attributes: StyleAttribute) -> Self {
        self.attribute.push(attributes);
        self
    }
    /// Get raw text content
    pub fn get_text(&self) -> &str {
        &self.text
    }
    /// Get slice of attributes
    pub fn get_attributes(&self) -> &[StyleAttribute] {
        &self.attribute
    }
    /// Get raw text content's length
    pub fn len(&self) -> usize {
        self.text.len()
    }
}
impl From<&str> for StyledText {
    fn from(text: &str) -> Self { Self::new(text.into()) }
}
impl From<String> for StyledText {
    fn from(text: String) -> Self { Self::new(text) }
}
