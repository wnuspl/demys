use std::fs::{self, DirEntry};
use std::fs::read_dir;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use crossterm::event::{KeyCode, KeyModifiers};
use crate::event::{EventPoster, Uuid};
use crate::plot::Plot;
use crate::style::{Canvas, StyleAttribute, StyledText, ThemeColor};
use crate::textedit::textwindow::TextWindow;
use crate::window::{TestWindow, Window, WindowEvent, WindowRequest};







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
    pub fn toggle(&mut self) {
        if !self.is_dir { return; }

        if self.is_open {
            self.close();
        } else {
            self.open();
        }
    }
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


    fn _map_line_child(&mut self, remaining: &mut usize) -> Option<&mut DirectoryRep> {
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
    pub fn map_line_child(&mut self, interface_line: usize) -> Option<&mut DirectoryRep> {
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

        let child_indent = format!("  {}", indent);

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
    line: usize,
    dir: DirectoryRep,
    poster: Option<EventPoster<WindowRequest,Uuid>>
}


// FILE SYSTEM TAB IMPL
// allows navigation of filesystem to open files
impl FSWindow {
    pub fn new(dir: PathBuf) -> FSWindow {
        FSWindow { line: 0, dir: dir.into(), poster: None }
    }
    fn input(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        match key {
            KeyCode::Char('j') => {
                let target = self.line+1;

                if self.dir.map_line_child(target).is_some() {
                    self.line = target;
                }
            }
            KeyCode::Char('k') => {
                if self.line > 0 {
                    self.line -= 1;
                }
            }
            KeyCode::Enter => {
                if let Some(item) = self.dir.map_line_child(self.line) {
                    if item.is_dir {
                        item.toggle();
                    } else {
                        // request creating new window
                        let text_window = TextWindow::from_file(item.dir.clone());
                        self.poster.as_mut().unwrap().post(WindowRequest::AddWindow(
                            Some(Box::new(text_window))
                        ));
                    }
                }
            }
            _ => ()
        }
    }
}





impl Window for FSWindow {
    fn name(&self) -> String {
        "Explorer".parse().unwrap()
    }
    fn draw(&self, canvas: &mut Canvas) {
        let text = self.dir.to_string();

        for (i, line) in text.split("\n").enumerate() {
            let mut styled = StyledText::new(line.to_string());

            if i == self.line {
                styled = styled.with(StyleAttribute::BgColor(ThemeColor::Yellow));
            }

            canvas.write_wrap(&styled);
            canvas.to_next_line();
        }
    }
    fn init(&mut self, poster: EventPoster<WindowRequest, Uuid>) {
        self.poster = Some(poster);
    }
    fn event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::Input {key, modifiers} => self.input(key, modifiers),
            _ => ()
        }
        self.poster.as_mut().unwrap().post(WindowRequest::Redraw);
    }
}