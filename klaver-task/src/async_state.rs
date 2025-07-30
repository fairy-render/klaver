use futures::{
    FutureExt,
    future::{Either, pending},
};
use klaver_util::{
    CaugthException,
    rquickjs::{self, CatchResultExt, Ctx, JsLifetime},
    throw, throw_if,
};
use pin_project_lite::pin_project;
use std::{
    cell::RefCell,
    rc::Rc,
    task::{Poll, ready},
};

use crate::{
    ResourceId, ResourceKind,
    cell::{ObservableCell, ObservableRefCell},
    exec_state::{AsyncId, ExecState},
    resource::{Resource, ResourceMap, TaskCtx},
    state,
    task::TaskStatus,
};

#[derive(Clone)]
pub struct AsyncState {
    pub(crate) exec: ExecState,
    pub(crate) exception: Rc<ObservableRefCell<Option<CaugthException>>>,
    pub(crate) resource_map: Rc<RefCell<ResourceMap>>,
}

unsafe impl<'js> JsLifetime<'js> for AsyncState {
    type Changed<'to> = AsyncState;
}

impl AsyncState {
    pub fn instance(ctx: &Ctx<'_>) -> rquickjs::Result<AsyncState> {
        match ctx.userdata::<Self>() {
            Some(ret) => Ok(ret.clone()),
            None => {
                let _ = throw_if!(
                    ctx,
                    ctx.store_userdata(AsyncState {
                        exec: Default::default(),
                        exception: Rc::new(ObservableRefCell::new(None)),
                        resource_map: Rc::new(RefCell::new(ResourceMap::new()))
                    })
                );

                Ok(ctx.userdata::<Self>().unwrap().clone())
            }
        }
    }

    /// Start a new async task
    pub fn push<'js, T: Resource<'js> + 'js>(
        ctx: &Ctx<'js>,
        resource: T,
    ) -> rquickjs::Result<Option<TaskHandle>> {
        let this = Self::instance(ctx)?;

        let exception = this.exception.clone();
        if exception.borrow().is_some() {
            return Ok(None);
        }

        let kind = this.resource_map.borrow_mut().register::<T>();
        let id = this.exec.create_task(None, kind);

        let task_ctx = TaskCtx::new(ctx.clone(), this.exec.clone(), kind, id, T::INTERNAL)?;

        if let Err(err) = task_ctx.init().catch(&task_ctx) {
            exception.update(move |mut m| *m = Some(err.into()));
            task_ctx.destroy()?;
            return Ok(None);
        }

        let kill = Rc::new(ObservableCell::new(false));

        let cell = kill.clone();

        let parent_id = this.exec.parent_id(id);
        let parent_state = if T::SCOPED {
            this.exec.task_status(parent_id)
        } else {
            None
        };

        ctx.spawn(async move {
            if exception.borrow().is_some() {
                task_ctx.destroy().ok();
                return;
            }

            let ctx = task_ctx.ctx().clone();

            let idle = if let Some(ob) = parent_state {
                Either::Left(WaitIdle {
                    state: WaitIdleState::Init,
                    cell: ob.clone(),
                })
            } else {
                Either::Right(pending::<()>())
            };

            let future = resource.run(task_ctx.clone());

            // Wait for either the task to finish or a uncaught exception
            futures::select! {
                ret = future.fuse() => {
                    if let Err(err) = ret.catch(&ctx) {
                        this.exec.task_status(id).as_mut().map(|m| {
                            m.set(TaskStatus::Failed)
                        });
                        exception.update(|mut m| *m = Some(err.into()));
                    } else {
                        this.exec.task_status(id).as_mut().map(|m| {
                            m.set(TaskStatus::Idle)
                        });
                        this.exec.wait_children(id).await;
                    }

                }
                _ = idle.fuse() => {
                    this.exec.task_status(id).as_mut().map(|m| {
                        m.set(TaskStatus::Idle)
                    });

                    // this.exec.wait_children(id).await;
                }
                _ = kill.subscribe().fuse() => {}
                _ = exception.subscribe().fuse() => { }
            }

            task_ctx.destroy().ok();
        });

        Ok(Some(TaskHandle { cell, id, kind }))
    }
}

impl AsyncState {
    pub(crate) async fn run<'js, T, U, R>(&self, ctx: Ctx<'js>, func: T) -> rquickjs::Result<R>
    where
        T: FnOnce(TaskCtx<'js>) -> U,
        U: Future<Output = rquickjs::Result<R>>,
    {
        let id = self.exec.create_task(None, ResourceKind::ROOT);

        let task_ctx = TaskCtx::new(
            ctx.clone(),
            self.exec.clone(),
            ResourceKind::ROOT,
            id,
            false,
        )?;

        self.exec.set_current(id);

        let ret = func(task_ctx);

        futures::select! {
            ret = ret.fuse() => {

                match ret.catch(&ctx) {
                    Ok(ret) => {
                        self.exec.task_status(id).unwrap().set(TaskStatus::Idle);
                        self.exec.wait_children(id).await;
                        self.exec.destroy_task(id);

                        if let Some(err) = &*self.exception.borrow() {
                            throw!(ctx, err)
                        }

                        Ok(ret)
                    }
                    Err(err) => {
                        let err: CaugthException = err.into();
                        self.exception.update(|mut m| *m = Some(err.clone()));
                        throw!(ctx, err)
                    }
                }
            }
            _ = self.exception.subscribe().fuse() => {
                if let Some(err) = &*self.exception.borrow() {
                    throw!(ctx, err);
                } else {
                    throw!(ctx, "Work did not finalize")
                }
            }
        }
    }
}

pub struct TaskHandle {
    id: AsyncId,
    kind: ResourceKind,
    cell: Rc<ObservableCell<bool>>,
}

impl TaskHandle {
    pub fn kill(self) {
        self.cell.set(true);
    }

    pub fn id(&self) -> AsyncId {
        self.id
    }

    pub fn kind(&self) -> ResourceKind {
        self.kind
    }

    pub fn is_running(&self) -> bool {
        self.cell.get()
    }
}

pin_project! {
    #[project = WaitIdleStateProj]
    enum WaitIdleState {
        Init,
        Waiting {
            #[pin]
            future: event_listener::EventListener
        }
    }
}

pin_project! {
    struct WaitIdle {
        #[pin]
        state: WaitIdleState,
        cell: Rc<ObservableCell<TaskStatus>>,
    }
}

impl Future for WaitIdle {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        loop {
            let mut this = self.as_mut().project();

            if this.cell.get() == TaskStatus::Idle {
                return Poll::Ready(());
            }

            match this.state.as_mut().project() {
                WaitIdleStateProj::Init => {
                    let future = this.cell.subscribe();
                    this.state.set(WaitIdleState::Waiting { future });
                }
                WaitIdleStateProj::Waiting { future } => {
                    ready!(future.poll(cx));
                    this.state.set(WaitIdleState::Init);
                }
            }
        }
    }
}
