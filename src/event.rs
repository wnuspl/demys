//! Event system that allows multiple posters but only one receiver

use std::cell::{Ref, RefCell};
use std::error::Error;
use std::rc::Rc;



/// Basic Uuid class
#[derive(Clone)]
#[derive(Debug)]
#[derive(Eq, Hash, PartialEq)]
pub struct Uuid(pub usize);
pub trait UniqueNext: Sized+Clone{
    fn next() -> Self;
}

impl UniqueNext for Uuid {
    fn next() -> Self {
        unsafe {
            unsafe_next_uuid()
        }
    }
}

static mut NEXT_UUID: usize = 0;
unsafe fn unsafe_next_uuid() -> Uuid {
    unsafe {
        NEXT_UUID = NEXT_UUID.wrapping_add(1);
        Uuid(NEXT_UUID)
    }
}



// EVENT SYSTEM


struct _EventReceiver<T: Sized, U: UniqueNext> {
    received: Vec<(U,T)>
}

/// Sole receiver of events from any number of posters
pub struct EventReceiver<T: Sized, U: UniqueNext> {
    receiver: Rc<RefCell<_EventReceiver<T,U>>>
}

impl <T: Sized, U: UniqueNext> EventReceiver<T,U> {
    pub fn new() -> Self {
        Self {
            receiver:  Rc::new(RefCell::new(_EventReceiver {
                received: Vec::new()
            }))
        }
    }
    /// Return vec of posted events, consuming them.
    pub fn poll(&mut self) -> Vec<(U,T)> {
       let vec = &mut self.receiver.borrow_mut().received;
        std::mem::replace(vec, Vec::new())
    }
    fn receive(&mut self, event: (U,T)) {
        self.receiver.borrow_mut().received.push(event);
    }
    /// Create a new poster.
    pub fn new_poster(&mut self) -> EventPoster<T,U> {
        EventPoster::new(self)
    }
}
impl <T: Sized, U:UniqueNext> Clone for EventReceiver<T,U> {
    /// Return shallow copy
    fn clone(&self) -> Self {
        Self {
            receiver: self.receiver.clone()
        }
    }
}





struct _EventPoster<T: Sized, U: UniqueNext> {
    receiver: EventReceiver<T,U>,
}

/// Used to post events to dedicated receiver, has a uuid.
#[derive(Clone)]
pub struct EventPoster<T: Sized, U: UniqueNext> {
    poster: Rc<RefCell<_EventPoster<T,U>>>,
    uuid: U
}
impl<T: Sized, U: UniqueNext> EventPoster<T,U> {
    fn new(receiver: &EventReceiver<T,U>) -> Self {
        Self {
            poster: Rc::new(RefCell::new(
                _EventPoster { receiver: receiver.clone() }
            )),
            uuid: U::next(),
        }
    }

    /// Post event to receiver's queue
    pub fn post(&mut self, event: T) {
        self.poster.borrow_mut().receiver.receive((self.uuid.clone(), event));
    }
    /// Post event to receiver's queue as different uuid
    pub fn post_lie(&mut self, event: T, uuid: U) {
        self.poster.borrow_mut().receiver.receive((uuid, event));
    }
    /// Get copy of receiver
    fn receiver_raw(&self) -> EventReceiver<T,U> {
        self.poster.borrow_mut().receiver.clone()
    }

    pub fn get_uuid(&self) -> &U { &self.uuid }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn uuid() {
        let uuid1 = Uuid::next();
        let uuid2 = Uuid::next();
        assert_ne!(uuid1, uuid2);
    }

    #[test]
    fn construction() {
        let mut receiver = EventReceiver::<String,Uuid>::new();
        let poster = receiver.new_poster();

        // poster has correct receiver
        assert!(std::ptr::eq(poster.receiver_raw().receiver.as_ref(), receiver.receiver.as_ref()));
    }

    #[test]
    fn single_post_receive() {
        let mut receiver = EventReceiver::<String,Uuid>::new();
        let mut poster = receiver.new_poster();

        let message = String::from("Hello World");
        poster.post(message.clone());

        let vec = receiver.poll();

        // correct number of events
        assert_eq!(vec.len(), 1);

        // events consumed
        assert_eq!(receiver.poll().len(), 0);

        // correct uuid
        assert_eq!(vec[0].0.clone(), poster.uuid);

        // correct message
        assert_eq!(vec[0].1, message.clone());
    }

    #[test]
    fn multiple_post_receive() {
        let mut receiver = EventReceiver::<String,Uuid>::new();
        let mut poster_vec = vec![receiver.new_poster(); 10];

        for p in poster_vec.iter_mut().take(5) {
            p.post("first".into());
        }

        // all 5 first events were receive
        let received1= receiver.poll();
        assert_eq!(received1.len(), 5);
        assert_eq!(receiver.poll().len(), 0);

        for p in poster_vec.iter_mut().skip(5) {
            p.post("second".into());
        }

        // all 5 first events were receive
        let received2 = receiver.poll();
        assert_eq!(received2.len(), 5);
        assert_eq!(receiver.poll().len(), 0);
    }

}