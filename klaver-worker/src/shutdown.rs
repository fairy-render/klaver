use std::{cell::Cell, rc::Rc, task::Poll};

use event_listener::EventListener;
use futures::{
    FutureExt,
    future::{Fuse, FusedFuture},
};
use pin_project_lite::pin_project;

pin_project! {
    pub struct Shutdown {
        #[pin]
        listener: Fuse<EventListener>,
        killed: Rc<Cell<bool>>,
    }
}

impl Shutdown {
    pub(crate) fn new(listener: EventListener, killed: Rc<Cell<bool>>) -> Shutdown {
        Shutdown {
            listener: listener.fuse(),
            killed,
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
        if this.killed.get() {
            Poll::Ready(())
        } else {
            this.listener.poll(cx)
        }
    }
}

impl FusedFuture for Shutdown {
    fn is_terminated(&self) -> bool {
        self.listener.is_terminated()
    }
}
