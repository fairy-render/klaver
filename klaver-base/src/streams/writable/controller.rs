use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use async_notify::Notify;
use rquickjs::{Ctx, JsLifetime, String, Value, class::Trace, prelude::Opt};

use crate::streams::queue_strategy::QueuingStrategy;

pub struct StreamReadyState {
    pub locked: bool,
    pub wait: Rc<Notify>,
}

impl<'js> Trace<'js> for StreamReadyState {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

// #[derive(Trace)]
// pub enum WritableStreamController<'js> {
//     Default(WritableStreamDefaultController<'js>),
// }

#[derive(Trace)]
enum ControllerState<'js> {
    Aborted(Option<String<'js>>),
    Failed(Value<'js>),
    Closed,
    Running,
}

#[rquickjs::class]
pub struct WritableStreamDefaultController<'js> {
    pub queue: RefCell<VecDeque<Value<'js>>>,
    pub wait: StreamReadyState,
    pub queing_strategy: QueuingStrategy<'js>,
    pub state: ControllerState<'js>,
}

impl<'js> Trace<'js> for WritableStreamDefaultController<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.queing_strategy.trace(tracer);
        self.queue.borrow().trace(tracer);
        self.wait.trace(tracer);
        self.state.trace(tracer);
    }
}

unsafe impl<'js> JsLifetime<'js> for WritableStreamDefaultController<'js> {
    type Changed<'to> = WritableStreamDefaultController<'to>;
}

impl<'js> WritableStreamDefaultController<'js> {
    pub fn is_ready(&self) -> bool {
        let max = self.queing_strategy.high_water_mark();
        if self.queue.borrow().len() >= (max as usize) {
            false
        } else {
            true
        }
    }

    pub fn close(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<()> {
        if !self.is_running() {
            todo!()
        }

        self.state = ControllerState::Closed;

        Ok(())
    }

    pub fn abort(&mut self, ctx: Ctx<'js>, reason: Option<String<'js>>) -> rquickjs::Result<()> {
        if !self.is_running() {
            todo!()
        }

        self.state = ControllerState::Aborted(reason);

        Ok(())
    }

    pub fn is_closed(&self) -> bool {
        matches!(self.state, ControllerState::Closed)
    }

    pub fn is_running(&self) -> bool {
        matches!(self.state, ControllerState::Running)
    }

    pub fn is_aborted(&self) -> bool {
        matches!(self.state, ControllerState::Aborted(_))
    }

    pub fn abort_reason(&self) -> Option<String<'js>> {
        match &self.state {
            ControllerState::Aborted(reason) => reason.clone(),
            _ => None,
        }
    }

    pub fn unlock(&mut self) {
        self.wait.locked = false;
    }

    pub fn lock(&mut self) {
        self.wait.locked = true;
    }

    pub async fn ready(&self) -> rquickjs::Result<()> {
        let max = self.queing_strategy.high_water_mark();

        loop {
            if (max as usize) > self.queue.borrow().len() {
                break;
            }

            self.wait.wait.notified().await
        }

        if !self.is_running() {
            todo!()
        }

        Ok(())
    }

    pub async fn write(&self, chunk: Value<'js>) -> rquickjs::Result<()> {
        self.ready().await?;

        self.queue.borrow_mut().push_back(chunk);

        Ok(())
    }
}
