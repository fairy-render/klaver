use std::cell::{Ref, RefCell, RefMut};

use event_listener::listener;

use crate::sync::Notify;

pub struct AsyncLock<T> {
    cell: RefCell<T>,
    event: Notify,
}

impl<T> AsyncLock<T> {
    pub async fn read(&self) -> ReadLockGuard<'_, T> {
        loop {
            if let Ok(inner) = self.cell.try_borrow() {
                return ReadLockGuard {
                    inner,
                    notify: &self.event,
                };
            }

            listener!(self.event.0 => listener);
            listener.await
        }
    }

    pub async fn write(&self) -> WriteLockGuard<'_, T> {
        loop {
            if let Ok(inner) = self.cell.try_borrow_mut() {
                return WriteLockGuard {
                    inner,
                    notify: &self.event,
                };
            }

            listener!(self.event.0 => listener);
            listener.await
        }
    }
}

pub struct ReadLockGuard<'a, T> {
    inner: Ref<'a, T>,
    notify: &'a Notify,
}

impl<'a, T> Drop for ReadLockGuard<'a, T> {
    fn drop(&mut self) {
        self.notify.notify();
    }
}

pub struct WriteLockGuard<'a, T> {
    inner: RefMut<'a, T>,
    notify: &'a Notify,
}

impl<'a, T> Drop for WriteLockGuard<'a, T> {
    fn drop(&mut self) {
        self.notify.notify();
    }
}
