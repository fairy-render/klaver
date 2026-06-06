use std::mem;

use rquickjs::class::Trace;

use crate::sync::{Listener, Notify};

pub struct Observable<T> {
    value: T,
    notify: Notify,
}

impl<T: PartialEq> Observable<T> {
    pub fn new(value: T) -> Observable<T> {
        Observable {
            value,
            notify: Notify::default(),
        }
    }

    pub fn set(&mut self, value: T) -> T {
        let updated = self.value != value;
        let old = mem::replace(&mut self.value, value);
        if updated {
            self.notify.notify();
        }
        old
    }

    pub fn get(&self) -> &T {
        &self.value
    }

    pub fn subscribe(&self) -> Listener {
        self.notify.listen()
    }
}

impl<T> std::ops::Deref for Observable<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> AsRef<T> for Observable<T> {
    fn as_ref(&self) -> &T {
        &self.value
    }
}

impl<'js, T: Trace<'js>> Trace<'js> for Observable<T> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.value.trace(tracer)
    }
}
