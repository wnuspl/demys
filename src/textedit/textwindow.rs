use std::ops::AddAssign;
use std::path::PathBuf;
use crossterm::event::{KeyCode, KeyModifiers};
use crate::plot::Plot;
use crate::style::{Canvas, StyleAttribute, StyledText, ThemeColor};
use crate::textedit::buffer::TextBuffer;
use crate::window::{ScrollableData, Scrollable, WindowRequest, Window};

enum Mode {
    Normal,
    Insert
}

struct TextWindowSettings {
    insert_color: ThemeColor,
    normal_color: ThemeColor,
    line_number_color: ThemeColor,
    dynamic_caret_color: bool,
    line_numbers: bool,
}

impl Default for TextWindowSettings {
    fn default() -> Self {
        Self {
            insert_color: ThemeColor::Blue,
            normal_color: ThemeColor::Gray,
            line_number_color: ThemeColor::Green,
            dynamic_caret_color: true,
            line_numbers: true
        }
    }
}

pub struct TextWindow {
    tb: TextBuffer,
    requests: Vec<WindowRequest>,
    name: String,
    scrollable_data: ScrollableData,
    mode: Mode,
    settings: TextWindowSettings,
}



// TEXT TAB IMPL
// Holds text buffers
impl TextWindow {
    pub fn new(tb: TextBuffer) -> TextWindow {
        let mut scrollable_data = ScrollableData::default();
        scrollable_data.scroll_margin = 1;
        TextWindow { tb, requests: Vec::new(), name: "[untitled]".into(), scrollable_data, mode: Mode::Normal, settings: TextWindowSettings::default()}
    }
    pub fn from_file(path: PathBuf) -> TextWindow {
        let name = path.file_name().unwrap().to_string_lossy().into();
        let tb = TextBuffer::from(path);
        let mut tw = Self::new(tb);
        tw.name = name;
        tw
    }

    fn insert_mode_input(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        match (key, modifiers) {
            (_, KeyModifiers::CONTROL) => match key {
                KeyCode::Char('[') => self.mode = Mode::Normal,
                _ => ()
            },

            (KeyCode::Backspace, _) => {
                self.tb.delete(1);
            }
            (KeyCode::Enter, _) => {
                self.tb.insert("\n");
            }
            (KeyCode::Char(ch), _) => {
                self.tb.insert(&ch.to_string());
            }
            (KeyCode::Esc, _) => self.mode = Mode::Normal,
            _ => ()
        }
    }
    fn normal_mode_input(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        match (key, modifiers) {
            (key, KeyModifiers::CONTROL) => match key {
                KeyCode::Char('s') => {
                    self.tb.save();
                }
                KeyCode::Char('l') => {
                    self.settings.line_numbers = !self.settings.line_numbers;
                }
                _ => ()
            }

            (KeyCode::Char('h'), _) => { self.tb.cursor_move_by(None,Some(-1)); }
            (KeyCode::Char('j'), _) => { self.tb.cursor_move_by(Some(1),None); }
            (KeyCode::Char('k'), _) => { self.tb.cursor_move_by(Some(-1),None); }
            (KeyCode::Char('l'), _) => { self.tb.cursor_move_by(None,Some(1)); }

            // insert mode transitions
            (KeyCode::Char('i'), _) => self.mode = Mode::Insert,
            (KeyCode::Char('I'), _) => {
                self.tb.cursor_start_of_line();
                self.mode = Mode::Insert;
            }
            (KeyCode::Char('a'), _) => {
                self.tb.cursor_move_by(None, Some(1));
                self.mode = Mode::Insert;
            }
            (KeyCode::Char('A'), _) => {
                self.tb.cursor_end_of_line();
                self.mode = Mode::Insert;
            }
            (KeyCode::Char('o'), _) => {
                self.tb.cursor_end_of_line();
                self.tb.insert("\n");
                self.mode = Mode::Insert;
            }
            _ => ()
        }
    }
}

impl Window for TextWindow {
    fn name(&self) -> String {
        let saved_symbol = if self.tb.saved { "" } else { "*" };
        format!("{}{}",saved_symbol,self.name)
    }
    fn input_bypass(&self) -> bool {
        match self.mode {
            Mode::Normal => false,
            Mode::Insert => true,
        }
    }
    fn input(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        // global controls
        match (key, modifiers) {
            _ => ()
        }

        // edit mode controls
        match self.mode {
            Mode::Insert => self.insert_mode_input(key, modifiers),
            Mode::Normal => self.normal_mode_input(key, modifiers),
        }

        self.requests.push(WindowRequest::Redraw);
    }

    fn draw(&self, canvas: &mut Canvas) {
        // get header content
        let mode_text;
        let mode_header_color;
        match self.mode {
            Mode::Normal => {
                mode_text = "NORMAL";
                mode_header_color = self.settings.normal_color;
            }
            Mode::Insert => {
                mode_text = "INSERT";
                mode_header_color = self.settings.insert_color;
            }
        }

        // create styled text
        let mode_header = StyledText::new(mode_text.to_string())
            .with(StyleAttribute::Color(mode_header_color))
            .with(StyleAttribute::Bold(true));

        let name_header = StyledText::new(self.name())
            .with(StyleAttribute::Bold(true));


        // write header
        canvas.move_to(Plot::new(0, canvas.last_col()-10));
        canvas.write(&mode_header);

        // canvas.move_to(Plot::new(canvas.last_row(), canvas.last_col()-name_header.len()));
        // canvas.write(&name_header);



        // write text and line number
        canvas.move_to(Plot::new(0,0));
        let text = format!("{}",self.tb);

        for (n, line) in text.split('\n').enumerate() {
            // line number
            if self.settings.line_numbers {
                let line_number = StyledText::new(format!("{:<3}", n+1))
                    .with(StyleAttribute::Color(self.settings.line_number_color));
                canvas.write(&line_number);
            }

            // text
            let content = StyledText::new(line.to_string());
            canvas.write(&content);
            canvas.to_next_line();
        }

        // write cursor
        let mut cursor = <Plot>::from(self.tb.cursor);
        if self.settings.line_numbers {
            cursor += Plot::new(0,3);
        }

        canvas.set_attribute(
            StyleAttribute::BgColor(
                if self.settings.dynamic_caret_color { mode_header_color } else { ThemeColor::Gray },
            ),
            cursor,
            cursor+Plot::new(0,1)
        );


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

