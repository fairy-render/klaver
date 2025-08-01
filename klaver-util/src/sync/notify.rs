use event_listener::{Event, EventListener};
use pin_project_lite::pin_project;

#[derive(Debug, Default)]
pub struct Notify(pub(crate) Event);

impl Notify {
    pub fn notify(&self) {
        self.0.notify(usize::MAX);
    }

    pub fn listen(&self) -> Listener {
        Listener {
            listener: self.0.listen(),
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
