use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use pin_project_lite::pin_project;

use crate::sync::{Listener, Notify};

pub struct RecevError {}

pub struct Sender<T> {
    notify: Notify,
    data: Rc<RefCell<Option<T>>>,
}
impl<T> Sender<T> {
    pub fn send(self, value: T) {
        *self.data.borrow_mut() = Some(value);
        self.notify.notify();
    }
}

pin_project! {
pub struct Receiver<T> {
  #[pin]
  listener: Listener,
  data: Weak<RefCell<Option<T>>>
}
}

impl<T> Future for Receiver<T> {
    type Output = T;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        todo!()
    }
}
