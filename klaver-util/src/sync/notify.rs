use std::{rc::Rc, task::Poll};

use event_listener::{Event, EventListener};
use futures::{Stream, ready};
use pin_project_lite::pin_project;

#[derive(Debug, Default)]
pub struct Notify(pub(crate) Rc<Event>);

impl Notify {
    pub fn notify(&self) {
        self.0.notify(usize::MAX);
    }

    pub fn listen(&self) -> Listener {
        Listener {
            listener: self.0.listen(),
        }
    }

    pub fn stream(&self) -> NotificationStream {
        NotificationStream {
            event: self.0.clone(),
            state: State::Init,
        }
    }

    pub fn total_listeners(&self) -> usize {
        self.0.total_listeners()
    }
}

pin_project! {
  pub struct Listener {
    #[pin]
    listener: EventListener
  }
}

impl Future for Listener {
    type Output = ();
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        self.project().listener.poll(cx)
    }
}

pin_project! {
    #[project = StateProj]
    enum State {
        Init,
        Wait {
            #[pin]
            future: EventListener
        }
    }
}

pin_project! {
    pub struct NotificationStream {
        event: Rc<Event>,
        #[pin]
        state: State
    }
}

impl Stream for NotificationStream {
    type Item = ();
    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        loop {
            let mut this = self.as_mut().project();

            match this.state.as_mut().project() {
                StateProj::Init => {
                    this.state.set(State::Wait {
                        future: this.event.listen(),
                    });
                }
                StateProj::Wait { future } => {
                    ready!(future.poll(cx));
                    return Poll::Ready(Some(()));
                }
            }
        }
    }
}
