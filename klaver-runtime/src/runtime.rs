use std::{
    rc::Rc,
    task::{Poll, ready},
};

use futures::{
    FutureExt,
    future::{Either, pending},
};
use klaver_util::{
    CaugthException,
    rquickjs::{self, CatchResultExt, Class, Ctx, JsLifetime, Value, class::Trace},
    sync::{ObservableCell, ObservableRefCell},
};
use pin_project_lite::pin_project;

use crate::{
    context::Context,
    id::AsyncId,
    listener::HookListeners,
    resource::{Resource, ResourceKind, ResourceMap},
    task::TaskStatus,
    task_manager::TaskManager,
};

#[rquickjs::class(crate = "rquickjs")]
pub(crate) struct Runtime<'js> {
    pub hooks: Class<'js, HookListeners<'js>>,
    pub manager: TaskManager,
    pub resource_map: ResourceMap,
    pub exception: Rc<ObservableRefCell<CaugthException>>,
}

unsafe impl<'js> JsLifetime<'js> for Runtime<'js> {
    type Changed<'to> = Runtime<'to>;
}

impl<'js> Trace<'js> for Runtime<'js> {
    fn trace<'a>(&self, tracer: klaver_util::rquickjs::class::Tracer<'a, 'js>) {
        self.hooks.trace(tracer);
    }
}

impl<'js> Runtime<'js> {
    pub fn push<T: Resource<'js> + 'js>(
        &mut self,
        ctx: &Ctx<'js>,
        resource: T,
    ) -> rquickjs::Result<TaskHandle> {
        let kind = self.resource_map.register::<T>();

        let id = self.manager.create_task(None, kind, !T::SCOPED);

        let kill = Rc::new(ObservableCell::new(false));
        let cell = kill.clone();

        let context = Context::new(ctx.clone(), &self, id, kind, T::INTERNAL);

        let root_id = self
            .manager
            .find_parent(id, |task| task.kind == ResourceKind::ROOT);

        let root_state = if T::SCOPED {
            root_id.and_then(|root_id| self.manager.task_status(root_id))
        } else {
            None
        };
        let exception = self.exception.clone();

        let ctx = ctx.clone();

        let manager = self.manager.clone();
        let hooks = self.hooks.clone();

        if !T::INTERNAL {
            self.hooks
                .borrow()
                .init(&ctx, id, kind, Some(self.manager.exectution_trigger_id()))?;
        }

        ctx.clone().spawn(async move {
            let resource_future = resource.run(context);

            let idle = if let Some(ob) = root_state {
                Either::Left(WaitIdle {
                    state: WaitIdleState::Init,
                    cell: ob.clone(),
                })
            } else {
                Either::Right(pending::<()>())
            };

            let exception_future = exception.subscribe();

            let status = futures::select! {
                ret = resource_future.fuse() => {
                    if let Err(err) = ret.catch(&ctx) {
                        *exception.borrow_mut() = err.into();
                        TaskStatus::Failed
                    } else {
                        TaskStatus::Idle
                    }
                },
                _ = kill.subscribe().fuse() => {
                    TaskStatus::Idle
                }
                _ = idle.fuse() => {
                    TaskStatus::Idle
                }
                _ = exception_future.fuse() => {
                    TaskStatus::Idle
                }
            };

            if let Some(state) = manager.task_status(id) {
                state.set(status);
            }

            if manager.destroy_task(id) && !T::INTERNAL {
                if let Err(err) = hooks.borrow().destroy(&ctx, id).catch(&ctx) {
                    *exception.borrow_mut() = err.into();
                }
            }
        });

        Ok(TaskHandle { id, kind, cell })
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
            future: klaver_util::sync::Listener
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
