use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    task::Poll,
};

use event_listener::EventListener;
use futures::{
    FutureExt,
    future::{Fuse, FusedFuture},
};
use pin_project_lite::pin_project;

use crate::state::State;

pin_project! {
    pub struct Shutdown {
        #[pin]
        listener: Fuse<EventListener>,
        killed: Rc<RefCell<State>>,
    }
}

impl Shutdown {
    pub(crate) fn new(listener: EventListener, killed: Rc<RefCell<State>>) -> Shutdown {
        Shutdown {
            listener: listener.fuse(),
            killed,
        }
    }

    pub fn is_killed(&self) -> bool {
        match &*self.killed.borrow() {
            State::Failed(_) | State::Stopped | State::Stopping => true,
            _ => false,
        }
    }
}

impl Future for Shutdown {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();

        match &*this.killed.borrow() {
            State::Failed(_) | State::Stopped | State::Stopping => Poll::Ready(()),
            _ => this.listener.poll(cx),
        }
    }
}

impl FusedFuture for Shutdown {
    fn is_terminated(&self) -> bool {
        self.listener.is_terminated()
    }
}
