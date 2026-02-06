use std::ptr::with_exposed_provenance;
use crossterm::event::KeyCode;
use crate::event::{EventPoster, Uuid};
use crate::plot::Plot;
use crate::popup::{PopUp, PopUpDimension, PopUpDimensionOption, PopUpPosition, PopUpPositionOption};
use crate::style::{Canvas, StyleAttribute, StyledText, ThemeColor};
use crate::window::{Window, WindowEvent, WindowRequest};



pub struct AlertSettings {
    pub margin: usize,
    pub border: bool,
    pub background: ThemeColor
}

impl Default for AlertSettings {
    fn default() -> AlertSettings {
        Self {
            margin: 2,
            border: true,
            background: ThemeColor::Background,
        }
    }
}

pub struct Alert {
    pub content: StyledText,
    pub options: Vec<(StyledText, Vec<WindowRequest>)>,
    pub settings: AlertSettings,
    pub event_poster: Option<EventPoster<WindowRequest,Uuid>>,
    pub current: usize
}

impl Default for Alert {
    fn default() -> Self {
        Self {
            content: StyledText::new("".to_string()),
            options: Vec::new(),
            settings: AlertSettings::default(),
            event_poster: None,
            current: 0
        }
    }
}

impl Window for Alert {
    fn init(&mut self, poster: EventPoster<WindowRequest, Uuid>) {
        self.event_poster = Some(poster);
    }
    fn draw(&self, canvas: &mut Canvas) {
        canvas.set_attribute(StyleAttribute::BgColor(self.settings.background),
             Plot::new(0,0),
             Plot::new(canvas.last_row(), canvas.last_col()+1)
        );
        // border
        if self.settings.border {
            let corner = StyledText::new("@".to_string())
                .with(StyleAttribute::Color(ThemeColor::Black))
                .with(StyleAttribute::Bold(true));
            let horizontal = StyledText::new("-".repeat(canvas.get_dim().col-2).to_string())
                .with(StyleAttribute::Color(ThemeColor::Black))
                .with(StyleAttribute::Bold(true));
            let vertical = StyledText::new("|".to_string())
                .with(StyleAttribute::Color(ThemeColor::Black))
                .with(StyleAttribute::Bold(true));

            // write corners
            canvas.write_at(&corner, Plot::new(0,0));
            canvas.write_at(&corner, Plot::new(canvas.last_row(),0));
            canvas.write_at(&corner, Plot::new(0,canvas.last_col()));
            canvas.write_at(&corner, Plot::new(canvas.last_row(),canvas.last_col()));


            // write borders
            canvas.write_at(&horizontal, Plot::new(0, 1));
            canvas.write_at(&horizontal, Plot::new(canvas.last_row(), 1));

            for row in 1..canvas.last_row() {
                canvas.write_at(&vertical, Plot::new(row, 0));
                canvas.write_at(&vertical, Plot::new(row, canvas.last_col()));
            }
        }

        // write content
        canvas.move_to(Plot::new(self.settings.margin, self.settings.margin));
        canvas.write(&self.content);

        // write options
        canvas.move_to(Plot::new(self.settings.margin+2, self.settings.margin));
        for (i, (option, _)) in self.options.iter().enumerate() {
            if i == self.current {
                let selected_option = option.clone()
                    .with(StyleAttribute::BgColor(ThemeColor::White));
               canvas.write(&selected_option);
            } else {
                canvas.write(&option);
            }

            canvas.write(&"  ".into());
        }

    }
    fn event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::Input { key, .. } => match key {
                KeyCode::Char(_) => self.current = (self.current + 1) % self.options.len(),
                KeyCode::Enter => {
                    let chosen = std::mem::take(&mut self.options[self.current].1);
                    for req in chosen {
                        self.event_poster.as_mut().unwrap().post(req);
                    }
                    self.event_poster.as_mut().unwrap().post(WindowRequest::RemoveSelfPopup);
                }
                _ => ()
            },
            _ => ()
        }

    }
}

impl PopUp for Alert {
    fn position(&self) -> PopUpPosition {
        PopUpPosition {
            row: PopUpPositionOption::Centered(
                -1 * (self.settings.margin + 3/2)  as isize
            ),
            col: PopUpPositionOption::Centered(
                -1 * (self.settings.margin + self.content.len()/2)  as isize
            )
        }
    }
    fn dimension(&self) -> PopUpDimension {
        PopUpDimension {
            row: PopUpDimensionOption::Fixed(3 + self.settings.margin*2),
            col: PopUpDimensionOption::Fixed(self.content.len() + self.settings.margin*2),
        }
    }
    fn local(&self) -> bool {
        false
    }
}