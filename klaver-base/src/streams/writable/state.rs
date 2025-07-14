use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use async_notify::Notify;
use rquickjs::{Ctx, String, Value, class::Trace};

use crate::streams::queue_strategy::QueuingStrategy;

#[derive(Trace)]
enum ControllerState<'js> {
    Aborted(Option<String<'js>>),
    Failed(Value<'js>),
    Closed,
    Running,
}

struct StreamData<'js> {
    pub queue: RefCell<VecDeque<Value<'js>>>,
    pub wait: Rc<Notify>,
    pub queing_strategy: QueuingStrategy<'js>,
    pub state: RefCell<ControllerState<'js>>,
}

impl<'js> StreamData<'js> {
    pub fn new(strategy: QueuingStrategy<'js>) -> StreamData<'js> {
        StreamData {
            queue: Default::default(),
            wait: Default::default(),
            queing_strategy: strategy,
            state: RefCell::new(ControllerState::Running),
        }
    }

    pub async fn ready(&self) -> rquickjs::Result<()> {
        let max = self.queing_strategy.high_water_mark();

        loop {
            if (max as usize) > self.queue.borrow().len() {
                break;
            }

            self.wait.notified().await
        }

        if !self.is_running() {
            todo!()
        }

        Ok(())
    }

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

        self.state.replace(ControllerState::Closed);

        self.wait.notify();

        Ok(())
    }

    pub fn abort(&mut self, ctx: Ctx<'js>, reason: Option<String<'js>>) -> rquickjs::Result<()> {
        if !self.is_running() {
            todo!()
        }

        self.state.replace(ControllerState::Aborted(reason));

        self.wait.notify();

        Ok(())
    }

    pub fn is_closed(&self) -> bool {
        matches!(*self.state.borrow(), ControllerState::Closed)
    }

    pub fn is_running(&self) -> bool {
        matches!(*self.state.borrow(), ControllerState::Running)
    }

    pub fn is_aborted(&self) -> bool {
        matches!(*self.state.borrow(), ControllerState::Aborted(_))
    }

    pub fn abort_reason(&self) -> Option<String<'js>> {
        match &*self.state.borrow() {
            ControllerState::Aborted(reason) => reason.clone(),
            _ => None,
        }
    }

    pub async fn push(&self, data: Value<'js>) -> rquickjs::Result<()> {
        self.ready().await?;
        self.queue.borrow_mut().push_back(data);
        self.wait.notify();
        Ok(())
    }
}
