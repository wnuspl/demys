use std::collections::HashMap;
use std::error::Error;
use std::future::Future;
use std::hash::Hash;
use crate::event::{EventPoster, EventReceiver, Uuid};
use crate::popup::PopUp;
use crate::style::Canvas;
use crate::window::{Window, WindowEvent, WindowRequest};

pub trait WindowContainer: Window {
    fn add_window(&mut self, window: Box<dyn Window>) -> Uuid;
    fn remove_window(&mut self, uuid: Uuid) -> Option<Box<dyn Window>>;
    fn add_popup(&mut self, popup: Box<dyn PopUp>) -> Uuid;
    fn remove_popup(&mut self, uuid: Uuid) -> Option<Box<dyn PopUp>>;
}


pub struct OrderedWindowContainer {
    windows: HashMap<Uuid, Box<dyn Window>>,
    window_order: Vec<Uuid>,

    popups: HashMap<Uuid, Box<dyn PopUp>>,
    popup_order: Vec<Uuid>,

    event_receiver: EventReceiver<WindowRequest,Uuid>,
    event_poster: Option<EventPoster<WindowRequest, Uuid>>,

    seen: Vec<(Uuid, WindowRequest)>,

    current: usize
}

impl OrderedWindowContainer {
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
            window_order: Vec::new(),
            popups: HashMap::new(),
            popup_order: Vec::new(),
            event_receiver: EventReceiver::new(),
            event_poster: None,
            seen: Vec::new(),
            current: 0
        }
    }
    pub fn get_receiver(&mut self) -> &mut EventReceiver<WindowRequest, Uuid> {
        &mut self.event_receiver
    }
    pub fn get_poster(&mut self) -> Option<&mut EventPoster<WindowRequest, Uuid>> {
        self.event_poster.as_mut()
    }
    pub fn set_poster(&mut self, sender: EventPoster<WindowRequest, Uuid>) {
        self.event_poster = Some(sender);
    }
    pub fn post(&mut self, event: WindowRequest) -> Result<(), Box<dyn Error>> {
        let poster = self.event_poster.as_mut().unwrap();
        poster.post(event);
        Ok(())
    }

    pub fn get_current(&self) -> usize { self.current }
    pub fn set_current(&mut self, current: usize) { self.current = current; }
    pub fn cycle_current(&mut self) {
        if self.window_order.len() == 0 { return; }
        // unfocus old
        if let Some(old) = self.get_from_order_mut(self.current) {
            old.event(WindowEvent::Unfocus);
        }
        self.current = (self.current + 1) % self.window_order.len();
        if let Some(cur) = self.get_from_order_mut(self.current) {
            cur.event(WindowEvent::Focus);
        }
    }

    pub fn get_from_order_mut(&mut self, i: usize) -> Option<&mut Box<dyn Window>> {
        if let Some(uuid) = self.window_order.get_mut(i) {
           self.windows.get_mut(uuid)
        } else {
            None
        }
    }
    pub fn get_from_order(&self, i: usize) -> Option<&Box<dyn Window>> {
        if let Some(uuid) = self.window_order.get(i) {
            self.windows.get(uuid)
        } else {
            None
        }
    }

    pub fn window_count(&self) -> usize {
        self.window_order.len()
    }
    pub fn window_order(&self) -> &Vec<Uuid> {
        &self.window_order
    }

    pub fn poll(&mut self) -> Vec<(Uuid, WindowRequest)> {
    self.seen.append(&mut self.event_receiver.poll());
        std::mem::take(&mut self.seen)
    }

    /// Checks for popups and input bypasses, replacing with None event to avoid double counts
    pub fn distribute_events(&mut self, event: &mut WindowEvent) {

        // to last popup
        let last_uuid = self.popup_order.last();
        if let Some(last_uuid) = last_uuid {
            if let Some(popup) = self.popups.get_mut(last_uuid) {
                popup.event(std::mem::replace(event, WindowEvent::None));
                self.event_poster.as_mut().unwrap().post(WindowRequest::Redraw);
                return;
            }
        }

        if let Some(window) = self.get_from_order_mut(self.current) {
            if window.input_bypass() {
                match event {
                    // only input events go through
                    event @ WindowEvent::Input {..} => {
                        window.event(std::mem::replace(event, WindowEvent::None));
                    }
                    _ => ()
                }
            }
        }
    }

    // returns processed events
    fn process_requests(&mut self) -> Vec<WindowRequest> {
        let mut processed = Vec::new();
        for (uuid, event) in self.poll() {
            match event {
                WindowRequest::AddWindow(window) => {
                    if let Some(window) = window {
                        self.add_window(window);
                        self.post(WindowRequest::Redraw);
                    }
                    processed.push(WindowRequest::AddWindow(None));
                }
                WindowRequest::AddPopup(popup) => {
                    if let Some(popup) = popup {
                        self.add_popup(popup);
                        self.post(WindowRequest::Redraw);
                    }
                    processed.push(WindowRequest::AddPopup(None));
                }
                WindowRequest::RemoveSelfWindow => {
                    self.remove_window(uuid.clone());
                    self.post(WindowRequest::Redraw);

                    processed.push(WindowRequest::RemoveSelfWindow);
                }
                WindowRequest::RemoveSelfPopup => {
                    self.remove_popup(uuid);
                    self.post(WindowRequest::Redraw);

                    processed.push(WindowRequest::RemoveSelfPopup);
                }
                WindowRequest::Command(command) => {
                    if let Some(window) = self.get_from_order_mut(self.get_current()) {
                        window.event(WindowEvent::Command(command.clone()));
                    }
                    self.post(WindowRequest::Redraw);
                    processed.push(WindowRequest::Command(command));
                }
                WindowRequest::Redraw => {
                    self.post(WindowRequest::Redraw);
                    processed.push(WindowRequest::Redraw);
                }
                event => self.seen.push((uuid, event)),
            }
        }
        processed
    }
}


impl Window for OrderedWindowContainer {
    fn init(&mut self, poster: EventPoster<WindowRequest, Uuid>) {
        self.event_poster = Some(poster);
    }
    fn event(&mut self, event: WindowEvent) {
        match event {
            // apply to all
            WindowEvent::TryQuit => {
                for window in self.windows.values_mut() {
                    window.event(WindowEvent::TryQuit);
                }
            },
            WindowEvent::Resize(dim) => {
                for window in self.windows.values_mut() {
                    window.event(WindowEvent::Resize(dim));
                }
            },

            // apply to current
            event => {
                if let Some(window) = self.get_from_order_mut(self.current) {
                    window.event(event);
                }
            }
        }
    }
    fn input_bypass(&self) -> bool {
        if let Some(window) = self.get_from_order(self.current) {
            window.input_bypass()
        } else {
            false
        }
    }
    fn draw(&self, canvas: &mut Canvas) {
        // draw popups!
        for uuid in self.popup_order.iter() {
            if let Some(popup) = self.popups.get(uuid) {
                let dim = popup.term_dim(canvas.get_dim());
                let pos = popup.term_pos(canvas.get_dim());

                let mut popup_canvas = Canvas::new(dim);
                popup.draw(&mut popup_canvas);
                canvas.add_child(popup_canvas, pos);
            }
        }
    }
    // fn name
    fn collect_requests(&mut self) -> Vec<WindowRequest> {
        for window in self.windows.values_mut() {
            window.collect_requests();
        }

        // if none
        if self.window_order.len() == 0 {
            self.event_poster.as_mut().unwrap().post(WindowRequest::RemoveSelfWindow);
        }

        self.process_requests()
    }
}

impl WindowContainer for OrderedWindowContainer {
    fn add_window(&mut self, mut window: Box<dyn Window>) -> Uuid {
        // init with receiver
        let receiver = self.event_receiver.new_poster();
        let uuid = receiver.get_uuid().clone();
        window.init(receiver);

        // add to map and order
        self.windows.insert(uuid.clone(), window);
        self.window_order.push(uuid.clone());

        uuid
    }
    fn remove_window(&mut self, uuid: Uuid) -> Option<Box<dyn Window>> {

        // remove uuid from order
        self.window_order.retain(|u| u != &uuid);

        self.cycle_current();


        // remove from map
        self.windows.remove(&uuid)
    }
    fn add_popup(&mut self, mut popup: Box<dyn PopUp>) -> Uuid {
        // init with receiver
        let receiver = self.event_receiver.new_poster();
        let uuid = receiver.get_uuid().clone();
        popup.init(receiver);

        // add to map and order
        self.popups.insert(uuid.clone(), popup);
        self.popup_order.push(uuid.clone());

        uuid
    }
    fn remove_popup(&mut self, uuid: Uuid) -> Option<Box<dyn PopUp>> {
        // remove uuid from order
        self.popup_order.retain(|u| u != &uuid);

        // remove from map
        self.popups.remove(&uuid)
    }
}




#[cfg(test)]
mod test {
    use crate::plot::Plot;
    use crate::style::Canvas;
    use crate::window::TestWindow;
    use crate::window::windowcontainer::{OrderedWindowContainer, WindowContainer};

    #[test]
    fn add_window() {
        let mut container = OrderedWindowContainer::new();

        let window = TestWindow::default();
        let uuid  = container.add_window(Box::new(window));

        // one window in container
        assert_eq!(container.windows.len(), 1);

        // window is returned on remove
        assert!(container.remove_window(uuid).is_some());

         // no windows now
        assert_eq!(container.windows.len(), 0);
    }
}