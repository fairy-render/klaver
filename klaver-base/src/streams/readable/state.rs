use klaver_util::sync::ObservableCell;
use rquickjs::{Ctx, JsLifetime, Value, class::Trace};

use crate::streams::queue_strategy::QueuingStrategy;

use super::queue::Queue;

#[derive(Trace, Debug, Clone, Copy)]
pub enum StreamState {
    Aborted,
    Failed,
    Closed,
    Running,
    // Done,
}

#[derive(Trace)]
#[rquickjs::class]
pub struct ReadableStreamData<'js> {
    pub queue: Queue<'js>,
    pub state: ObservableCell<StreamState>,
    pub reason: Option<Value<'js>>,
    pub locked: bool,
    pub resource_active: ObservableCell<bool>,
}

unsafe impl<'js> JsLifetime<'js> for ReadableStreamData<'js> {
    type Changed<'to> = ReadableStreamData<'to>;
}

impl<'js> ReadableStreamData<'js> {
    pub(crate) fn new(strategy: QueuingStrategy<'js>) -> ReadableStreamData<'js> {
        ReadableStreamData {
            queue: Queue::new(strategy),
            state: ObservableCell::new(StreamState::Running),
            reason: None,
            locked: false,
            resource_active: ObservableCell::new(true),
        }
    }

    pub fn is_readable(&self) -> bool {
        !(self.is_cancled() || self.is_failed() || (self.is_closed() && self.queue.is_empty()))
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }

    pub fn is_closed(&self) -> bool {
        matches!(self.state.get(), StreamState::Closed)
    }

    // pub fn is_done(&self) -> bool {
    //     matches!(self.state.get(), StreamState::Done)
    // }

    pub fn is_cancled(&self) -> bool {
        matches!(self.state.get(), StreamState::Aborted)
    }

    pub fn is_failed(&self) -> bool {
        matches!(self.state.get(), StreamState::Failed)
    }

    pub fn is_running(&self) -> bool {
        matches!(self.state.get(), StreamState::Running)
    }

    pub fn push(&mut self, ctx: &Ctx<'js>, chunk: Value<'js>) -> rquickjs::Result<()> {
        if !self.is_running() {
            todo!()
        }

        self.queue.push(ctx, chunk)?;

        Ok(())
    }

    pub fn close(&mut self, ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        if !self.is_locked() {
            todo!()
        }

        self.state.set(StreamState::Closed);
        // self.locked = false;

        Ok(())
    }

    pub fn fail(&mut self, ctx: &Ctx<'js>, reason: Option<Value<'js>>) -> rquickjs::Result<()> {
        if !self.is_locked() {
            todo!()
        }

        self.state.set(StreamState::Failed);
        self.reason = reason;
        // self.locked = false;

        Ok(())
    }

    pub fn cancel(&mut self, ctx: &Ctx<'js>, reason: Option<Value<'js>>) -> rquickjs::Result<()> {
        if !self.is_locked() {
            todo!()
        }

        self.state.set(StreamState::Aborted);
        self.reason = reason;
        // self.locked = false;

        Ok(())
    }
}
