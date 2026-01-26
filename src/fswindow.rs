use std::fs::{self, DirEntry};
use std::fs::read_dir;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use crossterm::event::{KeyCode, KeyModifiers};
use crate::{tab, GridPos};
use crate::style::{StyleItem, };
use crate::textwindow::TextWindow;
use crate::window::{Window, WindowRequest, Scrollable, ScrollableData, pad};
use std::fmt::Error;
use crate::style::StyleItem::Text;







struct DirectoryRep {
    children: Vec<DirectoryRep>,
    dir: PathBuf,
    name: String,

    is_dir: bool,
    is_open: bool
}


impl From<DirEntry> for DirectoryRep {
    fn from(value: DirEntry) -> Self {
        Self {
            children: Vec::new(),
            dir: value.path(),
            name: value.file_name().into_string().unwrap_or("-".into()),
            is_dir: value.file_type().unwrap().is_dir(),
            is_open: false
        }
    }
}

impl From<PathBuf> for DirectoryRep {
    fn from(value: PathBuf) -> Self {
        let mut dr = Self {
            children: Vec::new(),
            name: Path::file_name(&value).unwrap().to_str().unwrap_or("-").to_string(),
            dir: value,
            is_dir: true,
            is_open: false
        };

        dr.open();

        dr

    }
}

impl DirectoryRep {
    pub fn open(&mut self) -> std::io::Result<()> {
        if !self.is_dir { return Ok(()); }

        // clear previous children
        self.children = Vec::new();

        // add to children
        if let Ok(dir_iter) = read_dir(&self.dir) {
            for entry in dir_iter {
                if let Ok(entry) = entry {
                    self.children.push(entry.into());
                }
            }
        }

        self.is_open = true;

        
        Ok(())
    }
    pub fn close(&mut self) -> std::io::Result<()> {
        if !self.is_dir { return Ok(()); }

        self.children = Vec::new();
        self.is_open = false;

        Ok(())
    }
    pub fn get_child(&self, idx: usize) -> Option<&DirectoryRep> {
        self.children.get(idx)
    }
    pub fn get_child_mut(&mut self, idx: usize) -> Option<&mut DirectoryRep> {
        self.children.get_mut(idx)
    }


    fn _map_line_child(&mut self, remaining: &mut u16) -> Option<&mut DirectoryRep> {

        // base case, on selected currently
        if *remaining == 0 {
            return Some(self);
        }

        *remaining -= 1;


        for c in &mut self.children {
            let dr = c._map_line_child(remaining);

            if dr.is_some() {
                return dr;
            }
        }

        None

    }
    pub fn map_line_child(&mut self, interface_line: u16) -> Option<&mut DirectoryRep> {
        let mut r = interface_line;
        self._map_line_child(&mut r)
    } 





    fn to_string_with_indent(&self, indent: &str) -> String {
        // text file
        if !self.is_dir {
            return format!("{}{}", indent, self.name);
        }

        // directory
        if !self.is_open {
            return format!("{}> {}", indent, self.name);
        }

        // open directory
        let mut out = String::new();
        out += &format!("{}v {}\n", indent, self.name);

        let child_indent = format!("\t{}", indent);

        let mut iter = self.children.iter().peekable();
        while let Some(child) = iter.next() {
            out += &child.to_string_with_indent(&child_indent);
            if iter.peek().is_some() {
                out += "\n";
            }
        }

        out
    }
}




impl ToString for DirectoryRep {
    fn to_string(&self) -> String {
        self.to_string_with_indent("")
    }
}




pub struct FSWindow {
    line: u16,

    dir: DirectoryRep,

    scrollable_data: ScrollableData,




    requests: Vec<WindowRequest>,
}


// FILE SYSTEM TAB IMPL
// allows navigation of filesystem to open files
impl FSWindow {
    pub fn new(dir: PathBuf) -> FSWindow {
        let mut sd = ScrollableData::default();
        sd.scroll_margin = 2;
        sd.total_lines = 9999;

        
        FSWindow { line: 0, dir: dir.into(), requests: Vec::new(), scrollable_data: sd }
    }
}



impl Scrollable for FSWindow {
    fn get_data_mut(&mut self) -> &mut ScrollableData {
        &mut self.scrollable_data
    }
}



impl Window for FSWindow {
    fn name(&self) -> String {
        "Explorer".parse().unwrap()
    }
    fn on_resize(&mut self, dim: GridPos) {
        self.scrollable_data.screen_rows = dim.row;
    }
    fn style(&self, dim: GridPos) -> Vec<StyleItem> {

        let mut out = Vec::new();

        for (i, line) in self.dir.to_string().split("\n").enumerate() {

            // continue if not in viewport
            let i = i as u16;
            if i < self.scrollable_data.scroll_offset { continue; }
            if i > self.scrollable_data.scroll_offset+self.scrollable_data.screen_rows { break; }


            // highlight selected line
            if self.line == i {
                out.push(StyleItem::Color(Some(1)));
            }

            out.push(StyleItem::Text(line.to_string()));
            out.push(StyleItem::LineBreak);
            out.push(StyleItem::Color(None));
        }

        pad(&mut out, dim)
    }

    fn requests(&mut self) -> &mut Vec<WindowRequest> {
        &mut self.requests
    }


    fn input(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                let target = self.line as i16-1;
                if target < 0 { return; }

                let target = target as u16;

                if self.dir.map_line_child(target).is_none() { return; }

                self.scroll_to(target);
                self.line = target;
            },
            KeyCode::Down | KeyCode::Char('j') => {
                let target = self.line + 1;

                if self.dir.map_line_child(target).is_none() { return; }

                self.scroll_to(target);
                self.line = target;
            },
            KeyCode::Enter => {


                let targetted = self.dir.map_line_child(self.line).unwrap();

                if !targetted.is_dir {
                    // open new text tab
                    let opened = Box::new(TextWindow::from_file(targetted.dir.clone()));
                    self.requests.push(WindowRequest::AddWindow(opened));
                } else {
                    if targetted.is_open {
                        targetted.close();
                    } else {
                        targetted.open();
                    }
                }
            }
            _ => ()
        };

        self.requests.push(WindowRequest::Redraw);
    }
}