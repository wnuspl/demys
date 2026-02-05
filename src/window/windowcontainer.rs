use std::collections::HashMap;
use std::hash::Hash;
use crate::event::{EventPoster, EventReceiver, Uuid};
use crate::popup::PopUp;
use crate::style::Canvas;
use crate::window::{Window, WindowEvent, WindowRequest};

pub trait WindowContainer: Window {
    fn add_window(&mut self, window: Box<dyn Window>) -> Uuid;
    fn remove_window(&mut self, uuid: Uuid) -> Option<Box<dyn Window>>;
    fn add_popup(&mut self, popup: Box<dyn PopUp>) {}
    fn remove_popup(&mut self, uuid: Uuid) -> Option<Box<dyn PopUp>> { None }
}


pub struct TestContainer {
    windows: HashMap<Uuid, Box<dyn Window>>,
    window_order: Vec<Uuid>,
    event_receiver: EventReceiver<WindowRequest,Uuid>,
    event_poster: Option<EventPoster<WindowRequest, Uuid>>,
    current: usize
}

impl TestContainer {
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
            window_order: Vec::new(),
            event_receiver: EventReceiver::new(),
            event_poster: None,
            current: 0
        }
    }
    fn get_from_order_mut(&mut self, i: usize) -> Option<&mut Box<dyn Window>> {
        if let Some(uuid) = self.window_order.get_mut(i) {
           self.windows.get_mut(uuid)
        } else {
            None
        }
    }
    fn get_from_order(&self, i: usize) -> Option<&Box<dyn Window>> {
        if let Some(uuid) = self.window_order.get(i) {
            self.windows.get(uuid)
        } else {
            None
        }
    }
}


impl Window for TestContainer {
    fn init(&mut self, poster: EventPoster<WindowRequest, Uuid>) {
        self.event_poster = Some(poster);
    }
    fn event(&mut self, event: WindowEvent) {
        if let Some(window) = self.get_from_order_mut(self.current) {
           window.event(event);
        }
    }
}

impl WindowContainer for TestContainer {
    fn add_window(&mut self, mut window: Box<dyn Window>) -> Uuid {
        let receiver = self.event_receiver.new_poster();
        let uuid = receiver.get_uuid().clone();
        window.init(receiver);

        self.windows.insert(uuid.clone(), window);
        self.window_order.push(uuid.clone());

        uuid
    }
    fn remove_window(&mut self, uuid: Uuid) -> Option<Box<dyn Window>> {
        self.windows.remove(&uuid)
    }
}




#[cfg(test)]
mod test {
    use crate::plot::Plot;
    use crate::style::Canvas;
    use crate::window::TestWindow;
    use crate::window::windowcontainer::{TestContainer, WindowContainer};

    #[test]
    fn add_window() {
        let mut container = TestContainer::new();

        let window = TestWindow::default();
        let uuid  = container.add_window(Box::new(window));

        // one window in container
        assert_eq!(container.windows.len(), 1);

        // window is returned on remove
        assert!(container.remove_window(uuid).is_some());

         // no windows now
        assert_eq!(container.windows.len(), 0);
    }

    #[test]
    fn draw() {
        let mut container = TestContainer::new();

        let window = TestWindow::default();
        let uuid  = container.add_window(Box::new(window));

        // set to window
        container.current = 0;


        let mut container_canvas = Canvas::new(Plot::new(4,4));
        let mut direct_canvas = Canvas::new(Plot::new(4,4));

        // container.draw_uuid(uuid.clone(), &mut container_canvas);

        let window = container.remove_window(uuid).unwrap();
        window.draw(&mut direct_canvas);

        let mut buffer1 = Vec::new();
        let mut buffer2 = Vec::new();

        container_canvas.queue_write(&mut buffer1, Plot::new(0,0));
        direct_canvas.queue_write(&mut buffer2, Plot::new(0,0));

        // both canvas writes had the same effect
        assert_ne!(buffer1, buffer2);
    }
}