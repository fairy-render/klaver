use event_listener::EventListener;
use event_listener::{Event, listener};
use pin_project_lite::pin_project;
use rquickjs::{Ctx, JsLifetime, String, Value, class::Class, class::Trace};
use rquickjs::{Function, Promise};
use std::task::{Poll, ready};
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    rc::Rc,
    usize,
};

use crate::streams::queue::Entry;
use crate::streams::{queue::Queue, queue_strategy::QueuingStrategy};

#[derive(Trace, Debug)]
pub enum ControllerState<'js> {
    Aborted(Option<String<'js>>),
    Failed(Value<'js>),
    Closed,
    Running,
    Done,
}

#[rquickjs::class]
pub struct StreamData<'js> {
    pub queue: Queue<'js>,
    pub wait: Rc<Event>,
    pub state: ControllerState<'js>,
    pub locked: bool,
}

impl<'js> Trace<'js> for StreamData<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.queue.trace(tracer);
        self.state.trace(tracer);
    }
}

unsafe impl<'js> JsLifetime<'js> for StreamData<'js> {
    type Changed<'to> = StreamData<'to>;
}

impl<'js> StreamData<'js> {
    pub fn new(strategy: QueuingStrategy<'js>) -> StreamData<'js> {
        StreamData {
            queue: Queue::new(strategy),
            wait: Default::default(),
            state: ControllerState::Running,
            locked: false,
        }
    }

    pub fn is_ready(&self) -> bool {
        !self.queue.is_full()
    }

    pub fn close(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<()> {
        if !self.is_running() {
            todo!("close {:?}", self.state);
        }

        self.state = ControllerState::Closed;
        self.wait.notify(usize::MAX);

        Ok(())
    }

    pub fn abort(&mut self, ctx: Ctx<'js>, reason: Option<String<'js>>) -> rquickjs::Result<()> {
        if !self.is_running() {
            todo!()
        }

        self.state = ControllerState::Aborted(reason);
        self.queue.clear();
        self.wait.notify(usize::MAX);

        Ok(())
    }

    pub fn fail(&mut self, ctx: Ctx<'js>, error: Value<'js>) -> rquickjs::Result<()> {
        if !self.is_running() {
            todo!()
        }

        self.state = ControllerState::Failed(error);
        self.queue.clear();

        self.wait.notify(usize::MAX);

        Ok(())
    }

    pub fn finished(&mut self) -> rquickjs::Result<()> {
        assert!(self.is_closed());
        self.state = ControllerState::Done;
        self.wait.notify(usize::MAX);
        Ok(())
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }

    pub fn lock(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<()> {
        if !self.is_running() {
            todo!()
        }
        self.locked = true;
        self.wait.notify(usize::MAX);

        Ok(())
    }

    pub fn unlock(&mut self) {
        self.locked = false;
        self.wait.notify(usize::MAX);
    }

    pub fn is_closed(&self) -> bool {
        matches!(self.state, ControllerState::Closed)
    }

    pub fn is_failed(&self) -> bool {
        matches!(self.state, ControllerState::Failed(_))
    }

    pub fn is_running(&self) -> bool {
        matches!(self.state, ControllerState::Running)
    }

    pub fn is_aborted(&self) -> bool {
        matches!(self.state, ControllerState::Aborted(_))
    }

    pub fn is_finished(&self) -> bool {
        matches!(self.state, ControllerState::Done)
    }

    pub fn abort_reason(&self) -> Option<String<'js>> {
        match &self.state {
            ControllerState::Aborted(reason) => reason.clone(),
            _ => None,
        }
    }

    pub fn push(
        &mut self,
        ctx: Ctx<'js>,
        data: Value<'js>,
    ) -> rquickjs::Result<(Promise<'js>, Function<'js>, Function<'js>)> {
        let ret = self.queue.push(ctx, data)?;
        self.wait.notify(usize::MAX);
        Ok(ret)
    }

    pub fn pop(&mut self) -> Option<Entry<'js>> {
        let ret = self.queue.pop()?;
        self.wait.notify(usize::MAX);

        Some(ret)
    }
}

pin_project! {
    #[project = WaiteStateProj]
    enum WaitState {
        Wating {
            #[pin]
            listener: EventListener
        },
        Idle
    }
}

pin_project! {

    pub struct WaitDone<'js> {
        state: Class<'js, StreamData<'js>>,
        #[pin]
        listener: WaitState,
    }
}

impl<'js> WaitDone<'js> {
    pub fn new(state: Class<'js, StreamData<'js>>) -> WaitDone<'js> {
        WaitDone {
            state,
            listener: WaitState::Idle,
        }
    }
}

impl<'js> Future for WaitDone<'js> {
    type Output = rquickjs::Result<()>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        loop {
            let mut this = self.as_mut().project();

            match this.listener.as_mut().project() {
                WaiteStateProj::Wating { listener } => {
                    ready!(listener.poll(cx));
                    this.listener.set(WaitState::Idle);
                }
                WaiteStateProj::Idle => {
                    if this.state.borrow().is_failed() {
                    } else if this.state.borrow().is_aborted() {
                    } else if this.state.borrow().is_finished() {
                        return Poll::Ready(Ok(()));
                    } else {
                        this.listener.set(WaitState::Wating {
                            listener: this.state.borrow().wait.listen(),
                        });
                    }
                }
            }
        }
    }
}

pin_project! {

    pub struct WaitReady<'js> {
        state: Class<'js, StreamData<'js>>,
        #[pin]
        listener: WaitState,
    }
}

impl<'js> WaitReady<'js> {
    pub fn new(state: Class<'js, StreamData<'js>>) -> WaitReady<'js> {
        WaitReady {
            state,
            listener: WaitState::Idle,
        }
    }
}

impl<'js> Future for WaitReady<'js> {
    type Output = rquickjs::Result<()>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        loop {
            let mut this = self.as_mut().project();

            match this.listener.as_mut().project() {
                WaiteStateProj::Wating { listener } => {
                    ready!(listener.poll(cx));
                    this.listener.set(WaitState::Idle);
                }
                WaiteStateProj::Idle => {
                    if this.state.borrow().is_failed() {
                    } else if this.state.borrow().is_aborted() {
                    } else if this.state.borrow().is_ready() {
                        return Poll::Ready(Ok(()));
                    } else {
                        this.listener.set(WaitState::Wating {
                            listener: this.state.borrow().wait.listen(),
                        });
                    }
                }
            }
        }
    }
}
