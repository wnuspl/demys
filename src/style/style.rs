#[derive(Copy,Clone)]
pub enum Color {
    Primary,
    Blue,
    Magenta,
    Yellow,
    Green,
    Background,
}
impl From<Color> for crossterm::style::Color {
    fn from(color: Color) -> Self {
        match color {
            Color::Primary => crossterm::style::Color::Black,
            Color::Blue => crossterm::style::Color::Blue,
            Color::Magenta => crossterm::style::Color::Magenta,
            Color::Yellow => crossterm::style::Color::Yellow,
            Color::Green => crossterm::style::Color::Green,
            Color::Background => crossterm::style::Color::White,
        }
    }
}

#[derive(Copy,Clone)]
#[repr(usize)]
pub enum StyleAttribute {
    Color(Color),
    Bold(bool),
}
impl From<StyleAttribute> for usize {
    fn from(attr: StyleAttribute) -> Self {
        match attr {
            StyleAttribute::Color(_) => 0,
            StyleAttribute::Bold(_) => 1,
        }
    }
}

impl StyleAttribute {
    pub const COUNT: usize = 2;
}


pub struct StyledText {
    text: String,
    attribute: Vec<StyleAttribute>,
    wrap: bool
}
impl StyledText {
    pub fn new(text: String) -> Self {
        Self {
            text,
            attribute: Vec::new(),
            wrap: true,
        }
    }
    pub fn with(mut self, attributes: StyleAttribute) -> Self {
        self.attribute.push(attributes);
        self
    }
    pub fn get_text(&self) -> &str {
        &self.text
    }
    pub fn get_attributes(&self) -> &[StyleAttribute] {
        &self.attribute
    }
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
