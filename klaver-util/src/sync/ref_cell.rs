use std::cell::{Ref, RefCell};

use rquickjs::class::Trace;

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
    pub fn borrow(&self) -> Ref<T> {
        self.cell.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        RefMut {
            inner: self.cell.borrow_mut(),
            mutated: false,
            notify: &self.event,
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
