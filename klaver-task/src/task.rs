use event_listener::{Event, listener};
use klaver_util::{
    CaugthException,
    rquickjs::{
        self, CatchResultExt, Ctx, IntoJs, JsLifetime, String, Value, class::Trace,
        runtime::UserDataGuard,
    },
    throw_if,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use crate::{
    Shutdown,
    resource::{Resource, TaskCtx},
};

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

pub struct Task {
    resource: AsyncId,
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

#[derive(Debug, Clone)]
enum AsyncStatus {
    Running,
    Failed(CaugthException),
    ShuttingDown,
    Done,
}

pub struct AsyncState {
    shutdown: Event,
    events: Event,
    state: RefCell<ExecState>,
    killed: Rc<Cell<bool>>,
    status: RefCell<AsyncStatus>,
    resources_count: Cell<usize>,
}

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
                    ctx.store_userdata(AsyncState {
                        shutdown: Default::default(),
                        events: Default::default(),
                        state: RefCell::new(ExecState {
                            next_id: 1,
                            current_async_id: vec![root_id],
                            execution_id: vec![root_id]
                        }),
                        killed: Rc::new(Cell::new(false)),
                        status: RefCell::new(AsyncStatus::Running),
                        resources_count: Cell::new(0)
                    })
                );

                Ok(ctx.userdata().unwrap())
            }
        }
    }

    pub fn state(&self) -> &RefCell<ExecState> {
        &self.state
    }

    pub fn push<'js, T>(&self, ctx: Ctx<'js>, resource: T) -> rquickjs::Result<()>
    where
        T: Resource<'js> + 'js,
    {
        let listener = self.shutdown.listen();
        let shutdown = Shutdown::new(listener, self.killed.clone());

        let id = self.state.borrow_mut().next_id();
        let ty = String::from_str(ctx.clone(), "Async")?;

        let current_id = self.state.borrow().current_trigger_id();
        let task_ctx = TaskCtx::new(ctx.clone(), ty, current_id, id.clone())?;

        ctx.clone().spawn(async move {
            if let Err(err) = task_ctx.init() {}

            let ret = resource.run(task_ctx, shutdown).await.catch(&ctx);
            if let Err(err) = ret {
                eprintln!("{err}");
            }
        });

        Ok(())
    }

    // pub(crate) async fn wait(&self) -> Result<(), CaugthException> {
    //     loop {
    //         if let Some(error) = self.0.error.borrow_mut().take() {
    //             return Err(error);
    //         }

    //         if self.0.worker_count.get() == 0 && self.0.is_shutdown.get() {
    //             return Ok(());
    //         }

    //         listener!(self.0.events => events);

    //         events.await;

    //         if let Some(error) = self.0.error.borrow_mut().take() {
    //             return Err(error);
    //         }

    //         if self.0.worker_count.get() == 0 {
    //             return Ok(());
    //         }
    //     }
    // }
}
