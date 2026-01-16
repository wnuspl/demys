use std::error::Error;
use crate::window::Window;
use console::Key;
use console::Term;


pub struct WindowManager {
    layout: WindowLayout,
    windows: Vec<Window>,
    focused_window: usize,
    width: usize,
    height: usize,
}

// Maps windows in vec to locations/sizes in terminal
pub struct WindowLayout {
}

impl WindowManager {
    pub fn new(width: usize, height: usize, layout: WindowLayout) -> Self {
        Self {
            layout,
            windows: vec![Window::new()],
            focused_window: 0,
            width, height
        }
    }

    // propagates inputs down to Tab::input
    pub fn input(&mut self, key: Key) -> Result<(),String> {
        self.windows[self.focused_window].input(key)
    }


    pub fn display(&self, term: &mut Term) {
        let window = &self.windows[self.focused_window];
        let _ = term.clear_screen();
        let _ = term.write_line(&format!("{}", window.display(self.width, self.height)));
        if let Some(cursor_pos) = window.cursor_location() {
            let _ = term.show_cursor();
            let _ = term.move_cursor_to(cursor_pos.1, cursor_pos.0);
        } else {
            let _ = term.hide_cursor();
        }
    }
}
