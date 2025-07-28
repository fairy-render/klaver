use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

use event_listener::{Event, listener};
use klaver_util::{
    CaugthException,
    rquickjs::{
        self, CatchResultExt, Ctx, IntoJs, JsLifetime, String, Value, class::Trace,
        runtime::UserDataGuard,
    },
    throw_if,
};

use crate::{Resource, Shutdown, TaskCtx};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct AsyncId(u64);

impl<'js> Trace<'js> for AsyncId {
    fn trace<'a>(&self, _tracer: klaver_util::rquickjs::class::Tracer<'a, 'js>) {}
}

impl<'js> IntoJs<'js> for AsyncId {
    fn into_js(
        self,
        ctx: &klaver_util::rquickjs::Ctx<'js>,
    ) -> klaver_util::rquickjs::Result<klaver_util::rquickjs::Value<'js>> {
        Ok(Value::new_int(ctx.clone(), self.0 as _))
    }
}

#[derive(Debug, Clone)]
pub(crate) enum State {
    Running,
    Stopped,
    Failed(CaugthException),
    Stopping,
}

impl State {
    pub fn is_stopped(&self) -> bool {
        matches!(self, Self::Stopped)
    }

    pub fn is_stopping(&self) -> bool {
        matches!(self, Self::Stopping)
    }

    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }
}

pub struct ExecState {
    next_id: u64,
    current_async_id: Vec<AsyncId>,
    execution_id: Vec<AsyncId>,
}

impl ExecState {
    pub fn next_id(&mut self) -> AsyncId {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        AsyncId(id)
    }

    pub fn current_trigger_id(&self) -> AsyncId {
        self.current_async_id.last().copied().unwrap_or(AsyncId(0))
    }

    pub fn current_execution_id(&self) -> AsyncId {
        self.execution_id.last().copied().unwrap_or(AsyncId(0))
    }

    pub fn push(&mut self, id: AsyncId) {
        let parent = self.current_execution_id();
        self.current_async_id.push(id);
        self.execution_id.push(parent)
    }

    pub fn pop(&mut self) {
        self.current_async_id.pop();
        self.execution_id.pop();
    }
}

pub struct Task {}

struct Inner {
    events: Event,
    // Notifies when th
    shutdown: Event,
    uncaugth_exception: Rc<RefCell<Option<CaugthException>>>,
    resources: Cell<usize>,
    exec_state: RefCell<ExecState>,
}

pub struct AsyncState(Rc<Inner>);

unsafe impl<'js> JsLifetime<'js> for AsyncState {
    type Changed<'to> = AsyncState;
}

impl AsyncState {
    pub fn get<'a>(ctx: &'a Ctx<'_>) -> rquickjs::Result<UserDataGuard<'a, AsyncState>> {
        match ctx.userdata::<Self>() {
            Some(ret) => Ok(ret),
            None => {
                let root_id = AsyncId(0);

                let _ = throw_if!(
                    ctx,
                    ctx.store_userdata(AsyncState(Rc::new(Inner {
                        shutdown: Default::default(),
                        events: Default::default(),
                        state: Rc::new(RefCell::new(State::Running)),
                        exec_state: RefCell::new(ExecState {
                            next_id: 1,
                            current_async_id: vec![root_id],
                            execution_id: vec![root_id]
                        }),

                        resources: Cell::new(0)
                    })))
                );

                Ok(ctx.userdata().unwrap())
            }
        }
    }

    pub fn current_async_id(&self) -> AsyncId {
        self.0.exec_state.borrow().current_trigger_id()
    }

    pub fn execution_id(&self) -> AsyncId {
        self.0.exec_state.borrow().current_execution_id()
    }

    pub fn push<'js, T>(&self, ctx: Ctx<'js>, resource: T) -> rquickjs::Result<()>
    where
        T: Resource<'js> + 'js,
    {
        let listener = self.0.shutdown.listen();
        let shutdown = Shutdown::new(listener, self.0.state.clone());

        let id = self.0.exec_state.borrow_mut().next_id();
        let ty = String::from_str(ctx.clone(), "Async")?;

        let current_id = self.0.exec_state.borrow().current_trigger_id();
        let task_ctx = TaskCtx::new(ctx.clone(), ty, current_id, id.clone())?;

        self.0.resources.update(|m| m + 1);
        self.0.events.notify(usize::MAX);

        let inner = self.0.clone();

        ctx.clone().spawn(async move {
            if let Err(err) = task_ctx.init() {}

            let ret = resource.run(task_ctx, shutdown).await.catch(&ctx);

            inner.resources.update(|m| m - 1);

            if let Err(err) = ret {
                inner.state.replace(State::Failed(err.into()));
            }

            inner.events.notify(usize::MAX);
        });

        Ok(())
    }
}

impl AsyncState {
    // pub(crate) async fn wait(&self) -> Result<(), CaugthException> {
    //     loop {
    //         match &*self.0.state.borrow() {
    //             State::Stopping => {
    //                 if self.0.resources.get() == 0 {
    //                     self.0.state.replace(State::Stopped);
    //                     break Ok(());
    //                 }

    //                 listener!(self.0.events => listener);
    //                 listener.await
    //             }
    //             State::Running => {
    //                 // if self.0.resources.get() == 0 {
    //                 //     self.0.state.replace(State::Stopped);
    //                 //     break Ok(());
    //                 // }
    //                 // self.shutdown();

    //                 // listener!(self.0.events => listener);
    //                 // listener.await
    //             }
    //             State::Stopped => break Ok(()),
    //             State::Failed(caugth_exception) => break Err(caugth_exception.clone()),
    //         }

    //         // if self.0.resources.get() == 0 {
    //         //     self.0.state.replace(State::)
    //         // }
    //     }
    // }

    // pub(crate) fn shutdown(&self) {
    //     if !self.0.state.borrow().is_running() {
    //         return;
    //     }

    //     self.0.state.replace(State::Stopping);
    //     self.0.shutdown.notify(usize::MAX);
    //     self.0.events.notify(usize::MAX);
    // }

    pub(crate) fn enter(&self, id: AsyncId) {
        self.0.exec_state.borrow_mut().push(id);
    }

    pub(crate) fn leave(&self) {
        self.0.exec_state.borrow_mut().pop();
    }

    pub(crate) fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
