use crossterm::event::KeyCode;
use crate::event::{EventPoster, Uuid};
use crate::popup::{PopUp, PopUpDimension, PopUpDimensionOption, PopUpPosition, PopUpPositionOption};
use crate::style::Canvas;
use crate::window::{Window, WindowEvent, WindowRequest};

#[derive(Default)]
pub struct Command {
    cmd: String,
    poster: Option<EventPoster<WindowRequest,Uuid>>
}

impl Window for Command {
    fn init(&mut self, poster: EventPoster<WindowRequest, Uuid>) {
        self.poster = Some(poster);
    }
    fn draw(&self, canvas: &mut Canvas) {
       let text = format!(":{}", self.cmd);
        canvas.write(&text.into());
    }
    fn event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::Input {key, modifiers} => match (key, modifiers) {
                (KeyCode::Char(ch), _) => self.cmd += &ch.to_string(),
                (KeyCode::Backspace, _) => { self.cmd.remove(self.cmd.len()-1); },
                (KeyCode::Enter, _) => {
                    self.poster.as_mut().unwrap().post(WindowRequest::Command(self.cmd.clone()));
                    self.poster.as_mut().unwrap().post(WindowRequest::RemoveSelf);
                }
                (KeyCode::Esc, _) => {
                    self.poster.as_mut().unwrap().post(WindowRequest::RemoveSelf);
                }
                _ => ()
            }
            _ => ()
        }

        self.poster.as_mut().unwrap().post(WindowRequest::Redraw);
    }
}

impl PopUp for Command {
    fn position(&self) -> PopUpPosition {
        PopUpPosition {
            row: PopUpPositionOption::PositiveBound(1),
            col: PopUpPositionOption::NegativeBound(0)
        }
    }
    fn dimension(&self) -> PopUpDimension {
        PopUpDimension {
            row: PopUpDimensionOption::Fixed(1),
            col: PopUpDimensionOption::Percent(1.0)
        }
    }
    fn local(&self) -> bool {
        true
    }
}