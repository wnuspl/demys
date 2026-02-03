use crossterm::event::{KeyCode, KeyModifiers};
use crate::plot::Plot;
use crate::style::{Canvas, StyleAttribute, ThemeColor};
use crate::window::{Window, WindowRequest};

pub struct Draw {
    canvas: Canvas,
    requests: Vec<WindowRequest>
}

impl Draw {
    pub fn new(dim: Plot) -> Self {
        Self {
            canvas: Canvas::new(dim),
            requests: Vec::new()
        }
    }

    fn color_char(canvas: &mut Canvas) {
        let _ = canvas.set_attribute(
            StyleAttribute::BgColor(ThemeColor::Yellow),
            canvas.get_cursor(),
            canvas.get_cursor()+Plot::new(0,1)
        );
    }
}


impl Window for Draw {
    fn requests(&mut self) -> &mut Vec<WindowRequest> { self.requests.as_mut() }

    fn draw(&self, canvas: &mut Canvas) {
        canvas.merge_canvas(
            Plot::new(0,0),
            &self.canvas
        );

        canvas.move_to(self.canvas.get_cursor());
        Self::color_char(canvas);
    }

    fn input(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        let _ = match key {
            KeyCode::Up => self.canvas.move_to(
                self.canvas.get_cursor() - Plot::new(1,0)
            ),
            KeyCode::Down => self.canvas.move_to(
                self.canvas.get_cursor() + Plot::new(1,0)
            ),
            KeyCode::Left => self.canvas.move_to(
                self.canvas.get_cursor() - Plot::new(0,1)
            ),
            KeyCode::Right => self.canvas.move_to(
                self.canvas.get_cursor() + Plot::new(0,1)
            ),
            _ => Ok(())
        };

        match modifiers {
            KeyModifiers::SHIFT => Self::color_char(&mut self.canvas),
            _ => ()
        }

        self.requests.push(WindowRequest::Redraw);
    }
}