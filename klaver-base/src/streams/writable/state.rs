use event_listener::{Event, listener};
use rquickjs::{Ctx, String, Value, class::Trace};
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    rc::Rc,
    usize,
};

use crate::streams::{queue::Queue, queue_strategy::QueuingStrategy};

#[derive(Trace)]
pub enum ControllerState<'js> {
    Aborted(Option<String<'js>>),
    Failed(Value<'js>),
    Closed,
    Running,
    Done,
}

pub struct StreamData<'js> {
    pub queue: RefCell<Queue<'js>>,
    pub wait: Rc<Event>,
    pub state: RefCell<ControllerState<'js>>,
    pub locked: Cell<bool>,
}

impl<'js> Trace<'js> for StreamData<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.queue.borrow().trace(tracer);
        self.state.borrow().trace(tracer);
    }
}

impl<'js> StreamData<'js> {
    pub fn new(strategy: QueuingStrategy<'js>) -> StreamData<'js> {
        StreamData {
            queue: RefCell::new(Queue::new(strategy)),
            wait: Default::default(),
            state: RefCell::new(ControllerState::Running),
            locked: Cell::new(false),
        }
    }

    pub async fn ready(&self) -> rquickjs::Result<()> {
        loop {
            if !self.queue.borrow().is_full() {
                break;
            }

            listener!(self.wait => listener);
            listener.await;
        }

        if !self.is_running() {
            todo!()
        }

        Ok(())
    }

    pub fn is_ready(&self) -> bool {
        !self.queue.borrow().is_full()
    }

    pub fn close(&self, ctx: Ctx<'js>) -> rquickjs::Result<()> {
        if !self.is_running() {
            todo!()
        }

        self.state.replace(ControllerState::Closed);
        self.wait.notify(usize::MAX);

        Ok(())
    }

    pub fn abort(&self, ctx: Ctx<'js>, reason: Option<String<'js>>) -> rquickjs::Result<()> {
        if !self.is_running() {
            todo!()
        }

        self.state.replace(ControllerState::Aborted(reason));
        self.wait.notify(usize::MAX);

        Ok(())
    }

    pub fn fail(&self, ctx: Ctx<'js>, error: Value<'js>) -> rquickjs::Result<()> {
        if !self.is_running() {
            todo!()
        }

        self.state.replace(ControllerState::Failed(error));
        self.wait.notify(usize::MAX);

        Ok(())
    }

    pub async fn wait_done(&self, ctx: Ctx<'js>) -> rquickjs::Result<()> {
        loop {
            if self.is_aborted() {
                todo!()
            } else if self.is_failed() {
                todo!()
            } else if self.is_finished() {
                return Ok(());
            }

            listener!(self.wait => listener);

            listener.await;
        }
    }

    pub fn finished(&self) -> rquickjs::Result<()> {
        assert!(self.is_closed());
        self.state.replace(ControllerState::Done);
        self.wait.notify(usize::MAX);
        Ok(())
    }

    pub fn is_locked(&self) -> bool {
        self.locked.get()
    }

    pub fn lock(&self, ctx: Ctx<'js>) -> rquickjs::Result<()> {
        if !self.is_running() {
            todo!()
        }
        self.locked.set(true);
        self.wait.notify(usize::MAX);

        Ok(())
    }

    pub fn unlock(&self) {
        self.locked.set(false);
        self.wait.notify(usize::MAX);
    }

    pub fn is_closed(&self) -> bool {
        matches!(*self.state.borrow(), ControllerState::Closed)
    }

    pub fn is_failed(&self) -> bool {
        matches!(*self.state.borrow(), ControllerState::Failed(_))
    }

    pub fn is_running(&self) -> bool {
        matches!(*self.state.borrow(), ControllerState::Running)
    }

    pub fn is_aborted(&self) -> bool {
        matches!(*self.state.borrow(), ControllerState::Aborted(_))
    }

    pub fn is_finished(&self) -> bool {
        matches!(*self.state.borrow(), ControllerState::Done)
    }

    pub fn abort_reason(&self) -> Option<String<'js>> {
        match &*self.state.borrow() {
            ControllerState::Aborted(reason) => reason.clone(),
            _ => None,
        }
    }

    pub async fn push(&self, ctx: Ctx<'js>, data: Value<'js>) -> rquickjs::Result<()> {
        self.ready().await?;
        self.queue.borrow_mut().push(ctx, data)?;
        self.wait.notify(usize::MAX);
        Ok(())
    }

    pub async fn pop(&self) -> Option<Value<'js>> {
        let ret = self.queue.borrow_mut().pop()?;
        self.wait.notify(usize::MAX);

        Some(ret)
    }
}
