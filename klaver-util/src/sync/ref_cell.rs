use std::{
    cell::{Ref, RefCell},
    task::{Poll, ready},
};

use futures::Stream;
use pin_project_lite::pin_project;
use rquickjs::class::Trace;

use crate::sync::NotificationStream;

use super::{Listener, Notify};

pub struct ObservableRefCell<T> {
    event: Notify,
    cell: RefCell<T>,
}

impl<T> ObservableRefCell<T> {
    pub fn new(value: T) -> ObservableRefCell<T> {
        ObservableRefCell {
            event: Notify::default(),
            cell: RefCell::new(value),
        }
    }
}

impl<T> ObservableRefCell<T> {
    pub fn borrow(&self) -> Ref<'_, T> {
        self.cell.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        RefMut {
            inner: self.cell.borrow_mut(),
            mutated: false,
            notify: &self.event,
        }
    }

    pub fn wait_until<F>(&self, func: F) -> Wait<'_, F, T> {
        Wait {
            func,
            cell: &self.cell,
            stream: self.event.stream(),
        }
    }

    pub fn subscribe(&self) -> Listener {
        self.event.listen()
    }
}

impl<'js, T> Trace<'js> for ObservableRefCell<T>
where
    T: Trace<'js>,
{
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.cell.borrow().trace(tracer);
    }
}

pub struct RefMut<'a, T> {
    inner: std::cell::RefMut<'a, T>,
    mutated: bool,
    notify: &'a Notify,
}

impl<'a, T> std::ops::Deref for RefMut<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

impl<'a, T> std::ops::DerefMut for RefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.mutated = true;
        self.inner.deref_mut()
    }
}

impl<'a, T> Drop for RefMut<'a, T> {
    fn drop(&mut self) {
        if self.mutated {
            self.notify.notify();
        }
    }
}

pin_project! {
    pub struct Wait<'a, F, T> {
        func: F,
        cell: &'a RefCell<T>,
        #[pin]
        stream: NotificationStream,
    }

}

impl<'a, F, T> Future for Wait<'a, F, T>
where
    F: Fn(&T) -> bool,
{
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        loop {
            let this = self.as_mut().project();
            ready!(this.stream.poll_next(cx));
            if (this.func)(&this.cell.borrow()) {
                return Poll::Ready(());
            }
        }
    }
}
