use std::path::PathBuf;
use crossterm::event::{KeyCode, KeyModifiers};
use crate::plot::Plot;
use crate::style::{Canvas, StyleAttribute, StyledText, ThemeColor};
use crate::textedit::buffer::TextBuffer;
use crate::window::{ScrollableData, Scrollable, WindowRequest, Window};

pub struct TextWindow {
    tb: TextBuffer,
    requests: Vec<WindowRequest>,
    name: String,
    scrollable_data: ScrollableData,
    mode: bool,
}



// TEXT TAB IMPL
// Holds text buffers
impl TextWindow {
    pub fn new(tb: TextBuffer) -> TextWindow {
        let mut scrollable_data = ScrollableData::default();
        scrollable_data.scroll_margin = 1;
        TextWindow { tb, requests: Vec::new(), name: "".into(), scrollable_data, mode: true}
    }
    pub fn from_file(path: PathBuf) -> TextWindow {
        let name = path.file_name().unwrap().to_string_lossy().into();
        let tb = TextBuffer::from(path);
        let mut tw = Self::new(tb);
        tw.name = name;
        tw
    }
}


impl Window for TextWindow {
    fn name(&self) -> String {
        let saved_symbol = if self.tb.saved { "" } else { "*" };
        format!("{}{}",saved_symbol,self.name)
    }
    fn input(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        // global controls
        match (key, modifiers) {
            (key, KeyModifiers::CONTROL) => match key {
                KeyCode::Char('s') => {
                    self.tb.save();
                },
                KeyCode::Char('i') => {
                    self.mode = !self.mode;
                }
                _ => ()
            },
            _ => ()
        }

        // edit mode controls
        if !self.mode {
            match (key, modifiers) {
                (_, KeyModifiers::CONTROL) => (),

                (KeyCode::Backspace, _) => {
                    self.tb.delete(1);
                }
                (KeyCode::Enter, _) => {
                    self.tb.insert("\n");
                }
                (KeyCode::Char(ch), _) => {
                    self.tb.insert(&ch.to_string());
                }
                _ => ()
            }
        }

        self.requests.push(WindowRequest::Redraw);
    }
    fn draw(&self, canvas: &mut Canvas) {
        let mode_text;
        let mode_header_color;
        if self.mode {
            mode_text = "NORMAL";
            mode_header_color = ThemeColor::Yellow;
        } else {
            mode_text = "INSERT";
            mode_header_color = ThemeColor::Magenta;
        }

        let padding = canvas.get_dim().col - mode_text.len();
        let mode_header_text = format!("{}{}", mode_text, " ".repeat(padding));

        let mode_header = StyledText::new(mode_header_text)
            .with(StyleAttribute::BgColor(ThemeColor::Gray))
            .with(StyleAttribute::Color(mode_header_color))
            .with(StyleAttribute::Bold(true));

        canvas.write(&mode_header);


        let text = format!("{}",self.tb);


        let mut prev = Plot::new(0,0);
        for (n, line) in text.split('\n').enumerate() {
            let line_number = StyledText::new(format!("{} ", n+1))
                .with(StyleAttribute::Color(ThemeColor::Green));
            let content = StyledText::new(line.to_string());
            canvas.write(&line_number);
            canvas.write(&content);
            prev = canvas.get_cursor();
            canvas.next_line();
        }

        canvas.move_to(prev);

        canvas.show_cursor(true);
    }

    fn requests(&mut self) -> &mut Vec<WindowRequest> {
        &mut self.requests
    }
}


impl Scrollable for TextWindow {
    fn get_data_mut(&mut self) -> &mut ScrollableData {
        &mut self.scrollable_data
    }
}

