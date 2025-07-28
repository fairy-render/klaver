use std::{
    cell::{Cell, Ref, RefCell, RefMut},
    usize,
};

use event_listener::{Event, EventListener};

pub struct ObservableRefCell<T> {
    event: Event,
    cell: RefCell<T>,
}

impl<T> ObservableRefCell<T> {
    pub fn new(value: T) -> ObservableRefCell<T> {
        ObservableRefCell {
            event: Event::default(),
            cell: RefCell::new(value),
        }
    }
}

impl<T> ObservableRefCell<T> {
    pub fn borrow(&self) -> Ref<T> {
        self.cell.borrow()
    }

    pub fn update<F: FnOnce(RefMut<'_, T>) -> U, U>(&self, func: F) -> U {
        let ret = func(self.cell.borrow_mut());
        self.event.notify(usize::MAX);
        ret
    }

    pub fn subscribe(&self) -> EventListener {
        self.event.listen()
    }
}

pub struct ObservableCell<T> {
    event: Event,
    cell: Cell<T>,
}

impl<T> ObservableCell<T> {
    pub fn new(value: T) -> ObservableCell<T> {
        ObservableCell {
            event: Event::default(),
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
        self.event.notify(usize::MAX);
    }

    pub fn subscribe(&self) -> EventListener {
        self.event.listen()
    }
}
