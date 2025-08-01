use std::cell::Cell;

use rquickjs::class::Trace;

use super::{Listener, Notify};

pub struct ObservableCell<T> {
    event: Notify,
    cell: Cell<T>,
}

impl<T> ObservableCell<T> {
    pub fn new(value: T) -> ObservableCell<T> {
        ObservableCell {
            event: Notify::default(),
            cell: Cell::new(value),
        }
    }
}

impl<T: Copy> ObservableCell<T> {
    pub fn get(&self) -> T {
        self.cell.get()
    }

    pub fn set(&self, value: T) {
        self.cell.set(value);
        self.event.notify();
    }

    pub fn subscribe(&self) -> Listener {
        self.event.listen()
    }
}

impl<'js, T: Copy> Trace<'js> for ObservableCell<T>
where
    T: Trace<'js>,
{
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.cell.get().trace(tracer);
    }
}
