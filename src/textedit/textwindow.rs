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
        TextWindow { tb, requests: Vec::new(), name: "[untitled]".into(), scrollable_data, mode: true}
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
    fn input_bypass(&self) -> bool {
        !self.mode
    }
    fn input(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        // global controls
        match (key, modifiers) {
            (key, KeyModifiers::CONTROL) => match key {
                KeyCode::Char('i') => {
                    self.mode = !self.mode;
                }
                _ => ()
            }
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
        } else {
            match (key, modifiers) {
                (key, KeyModifiers::CONTROL) => match key {
                    KeyCode::Char('s') => {
                        self.tb.save();
                    }
                    _ => ()
                }

                (KeyCode::Char('h'), _) => { self.tb.cursor_move_by(None,Some(-1)); }
                (KeyCode::Char('j'), _) => { self.tb.cursor_move_by(Some(1),None); }
                (KeyCode::Char('k'), _) => { self.tb.cursor_move_by(Some(-1),None); }
                (KeyCode::Char('l'), _) => { self.tb.cursor_move_by(None,Some(1)); }
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


        let mode_header = StyledText::new(mode_text.to_string())
            .with(StyleAttribute::Color(mode_header_color))
            .with(StyleAttribute::Bold(true));

        let name_header = StyledText::new(self.name())
            .with(StyleAttribute::Bold(true));

        canvas.write(&mode_header);

        let right_side = canvas.get_dim().col - name_header.len();
        canvas.write_at(&name_header, Plot::new(0, right_side));
        canvas.next_line();

        canvas.set_attribute(StyleAttribute::BgColor(ThemeColor::Gray),0,canvas.get_dim().col);


        let text = format!("{}",self.tb);


        for (n, line) in text.split('\n').enumerate() {
            let line_number = StyledText::new(format!("{:<3}", n+1))
                .with(StyleAttribute::Color(ThemeColor::Green));
            let content = StyledText::new(line.to_string());
            canvas.write(&line_number);
            canvas.write(&content);
            canvas.next_line();
        }

        let cursor = <Plot>::from(self.tb.cursor) + Plot::new(1,3);
        canvas.move_to(cursor);


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

