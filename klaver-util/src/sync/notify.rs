use std::{
    cell::RefCell,
    rc::{Rc, Weak},
    task::{Poll, Waker},
};

use pin_project_lite::pin_project;
use slotmap::{DefaultKey, SlotMap};

#[derive(Debug, Default)]
struct Inner {
    wakers: SlotMap<DefaultKey, Waker>,
    listeners: usize,
}

#[derive(Debug, Default)]
pub struct Notify(Rc<RefCell<Inner>>);

impl Notify {
    pub fn notify(&self) {
        let mut inner = self.0.borrow_mut();
        inner.listeners = 0;
        for (_, waker) in inner.wakers.drain() {
            waker.wake();
        }
    }

    pub fn listen(&self) -> Listener {
        self.0.borrow_mut().listeners += 1;
        Listener {
            wakers: State::Init(Some(Rc::downgrade(&self.0))),
        }
    }
}

impl Drop for Notify {
    fn drop(&mut self) {
        self.notify();
    }
}

enum State {
    Init,
    Wait(DefaultKey),
    Done,
}

pin_project! {
  pub struct Listener {
    state: State,
    wakers:Weak<RefCell<Inner>>
  }
}

impl Future for Listener {
    type Output = ();
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();
        match &mut *this.wakers {
            State::Init(wakers) => {
                let Some(wakers) = wakers.take().unwrap().upgrade() else {
                    return Poll::Ready(());
                };
                let key = wakers.borrow_mut().wakers.insert(cx.waker().clone());

                *this.wakers = State::Wait(key);

                Poll::Pending
            }
            State::Wait(_key) => {
                *this.wakers = State::Done;
                Poll::Ready(())
            }
            State::Done => {
                panic!("Poll after done")
            }
        }
    }
}
