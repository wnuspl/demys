use crossterm::event::KeyCode;
use crate::GridPos;
use crate::style::StyleItem;
use crate::window::{Window, WindowRequest};


// style and functionality settings
#[derive(Default,Copy,Clone)]
pub struct TabConfig {
    show_tabs: bool,
    header_height: u16,
    tab_width: u16,
}



// tab manager, combines multiple windows into one
pub struct TabWindow {
    tabs: Vec<Box<dyn Window>>,
    requests: Vec<WindowRequest>,
    current_tab: usize,
    config: TabConfig
}

impl TabWindow {
    pub fn new(tabs: Vec<Box<dyn Window>>) -> Self {
        let mut config = TabConfig::default();
        config.header_height = 3;
        config.tab_width = 16;
        Self {
            tabs,
            requests: Vec::new(),
            current_tab: 0,
            config
        }
    }
    pub fn add_tab(&mut self, tab: Box<dyn Window>) {
        self.tabs.push(tab);
    }

    pub fn cycle_tab(&mut self) {
        self.current_tab += 1;
        if self.current_tab >= self.tabs.len() { self.current_tab = 0; }

        self.tabs[self.current_tab].on_focus();
        self.requests.push(WindowRequest::Clear);
        self.requests.push(WindowRequest::Redraw);
    }
}

impl Window for TabWindow {
    fn style(&self, dim: GridPos) -> Vec<StyleItem> {
        // HEADER
        let mut out = Vec::new();
        let tab_count = self.tabs.len();
        let mut top = String::new();
        let mut mid = String::new();
        let mut bar = String::new();

        let tab_body_width = self.config.tab_width-2;

        for i in 0..tab_count {
            top += "┌";
            mid += "│";

            for _ in 0..tab_body_width { top += "─"; }

            mid += &format!(" {:<w$}", self.tabs[i].name(), w=tab_body_width as usize -1);

            top += "┐";
            mid += "│";
        }

        for _ in 0..dim.col { bar += "─"; }
        out.push(StyleItem::Text(top));
        out.push(StyleItem::LineBreak);
        out.push(StyleItem::Text(mid));
        out.push(StyleItem::LineBreak);
        out.push(StyleItem::Text(bar));
        out.push(StyleItem::LineBreak);

        // remove rows used in header
        let dim = (dim.row-self.config.header_height, dim.col).into();

        // get body from current tab
        let current = &self.tabs[self.current_tab];
        out.append(&mut current.style(dim));

        out
    }
    fn input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Tab => {
                self.cycle_tab();
            }

            // remove current tab, create new window
            KeyCode::Insert => {
                // can't remove last tab
                if self.tabs.len() == 1 { return; }

                let removed = self.tabs.remove(self.current_tab);
                self.cycle_tab();

                self.requests.push(WindowRequest::AddWindow(removed));
            }

            _ => {
                self.tabs[self.current_tab].input(key);
            }
        }
    }
    fn requests(&mut self) -> &mut Vec<WindowRequest> {
        let mut current_req = self.tabs[self.current_tab].poll();


        // Get replacement/add request indices
        let mut replace = Vec::new();
        let mut add = Vec::new();
        for (i, r) in current_req.iter_mut().enumerate() {
            match &r {
                WindowRequest::ReplaceWindow(w) => {
                    replace.push(i)
                }
                WindowRequest::AddWindow(w) => {
                    add.push(i)
                }
                WindowRequest::Cursor(loc) => {
                    if let Some(mut loc) = loc.clone() {
                        loc = loc+(self.config.header_height, 0).into();
                        *r = WindowRequest::Cursor(Some(loc));
                    }
                }
                _ => ()
            }
        }

        // remove from requests and update self
        for i in replace {
            let r = current_req.remove(i);
            if let WindowRequest::ReplaceWindow(w) = r {
                self.tabs[self.current_tab] = w;
            }
        }
        for i in add {
            let r = current_req.remove(i);
            if let WindowRequest::AddWindow(w) = r {
                self.add_tab(w);
            }
        }





        // return self requests, and remaining from focused tab
        self.requests.append(&mut current_req);
        &mut self.requests
    }

    fn on_resize(&mut self, dim: GridPos) {
        for t in &mut self.tabs {
            t.on_resize((dim.row-self.config.header_height, dim.col).into());
        }
    }
}