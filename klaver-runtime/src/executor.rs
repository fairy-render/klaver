use std::{
    cell::RefCell,
    rc::Rc,
    task::{Poll, ready},
};

use futures::{
    FutureExt,
    future::{Either, pending},
};
use klaver_util::{
    CaugthException,
    rquickjs::{
        self, CatchResultExt, Class, Ctx, Function, IntoJs, JsLifetime, Value, class::Trace,
        function::Args, prelude::Rest,
    },
    sync::{ObservableCell, ObservableRefCell},
    throw,
};
use pin_project_lite::pin_project;

use crate::{
    AsyncId, Resource,
    context::Context,
    listener::HookListeners,
    resource::{ResourceKind, ResourceMap},
    runtime::Runtime,
    task::TaskStatus,
    task_manager::TaskManager,
};

pub struct TaskExecutor<'js> {
    hooks: Class<'js, HookListeners<'js>>,
    manager: TaskManager,
    exception: Rc<ObservableRefCell<Option<CaugthException>>>,
    resource_map: Rc<RefCell<ResourceMap>>,
}

impl<'js> Trace<'js> for TaskExecutor<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.hooks.trace(tracer);
    }
}

impl<'js> TaskExecutor<'js> {
    pub fn from_ctx(ctx: &Ctx<'js>) -> rquickjs::Result<TaskExecutor<'js>> {
        let runtime = Runtime::from_ctx(ctx)?;
        Ok(Self::new(&*runtime.borrow()))
    }

    pub fn new(runtime: &Runtime<'js>) -> TaskExecutor<'js> {
        TaskExecutor {
            hooks: runtime.hooks.clone(),
            manager: runtime.manager.clone(),
            exception: runtime.exception.clone(),
            resource_map: runtime.resource_map.clone(),
        }
    }

    pub fn manager(&self) -> &TaskManager {
        &self.manager
    }

    pub fn hooks(&self) -> Class<'js, HookListeners<'js>> {
        self.hooks.clone()
    }

    pub async fn run_async<T, R>(
        &self,
        ctx: &Ctx<'js>,
        kind: ResourceKind,
        runner: T,
    ) -> rquickjs::Result<R>
    where
        T: AsyncFnOnce(Context<'js>) -> rquickjs::Result<R>,
    {
        let id = self
            .manager
            .create_task(None, kind, kind == ResourceKind::ROOT, false);
        let parent_id = self.manager.parent_id(id);

        self.hooks.borrow().init(ctx, id, kind, Some(parent_id))?;

        let context = Context {
            hooks: self.hooks.clone(),
            id,
            tasks: self.manager.clone(),
            exception: self.exception.clone(),
            ctx: ctx.clone(),
            internal: false,
        };

        self.manager.set_current(id);

        let work_future = (runner)(context);

        let ret = futures::select! {
          ret = work_future.fuse() => {
            if let Some(state) = self.manager.task_status(id) {
              state.set(TaskStatus::Idle);
            }
            ret
          }
          _ = self.exception.subscribe().fuse() => {
            todo!("Exception")
          }
        };

        if kind == ResourceKind::ROOT {
            self.manager.wait_children(id).await;
        }

        self.manager.destroy_task(id, ctx, &self.hooks, true)?;

        if let Some(found) = self.exception.borrow().clone() {
            throw!(ctx, found);
        }

        ret
    }

    pub fn run<T, R>(&self, ctx: &Ctx<'js>, kind: ResourceKind, runner: T) -> rquickjs::Result<R>
    where
        T: FnOnce(Context<'js>) -> rquickjs::Result<R>,
    {
        let id = self
            .manager
            .create_task(None, kind, kind == ResourceKind::ROOT, false);
        let parent_id = self.manager.parent_id(id);

        self.hooks.borrow().init(ctx, id, kind, Some(parent_id))?;

        let context = Context {
            hooks: self.hooks.clone(),
            id,
            tasks: self.manager.clone(),
            exception: self.exception.clone(),
            ctx: ctx.clone(),
            internal: false,
        };

        // let current_id = self.manager.exectution_trigger_id();

        // // self.manager.set_current(id);

        let ret = (runner)(context);

        // self.manager.set_current(current_id);

        if let Some(state) = self.manager.task_status(id) {
            state.set(TaskStatus::Idle);
        }

        if kind == ResourceKind::ROOT {
            let manager = self.manager.clone();
            let hooks = self.hooks.clone();
            let cloned_ctx = ctx.clone();
            ctx.spawn(async move {
                manager.wait_children(id).await;
                // if manager.destroy_task(id, &cloned_ctx, &hooks).ok() {
                //     // hooks.borrow().destroy(&cloned_ctx, id).ok();
                // }
                manager.destroy_task(id, &cloned_ctx, &hooks, true).ok();
            });
        } else {
            if self.manager.destroy_task(id, ctx, &self.hooks, true)? {
                self.hooks.borrow().destroy(ctx, id)?;
            }
        }

        ret
    }

    pub fn push<T: Resource<'js> + 'js>(
        &self,
        ctx: &Ctx<'js>,
        resource: T,
    ) -> rquickjs::Result<TaskHandle> {
        let kind = self.resource_map.borrow_mut().register::<T>();

        let id = self
            .manager
            .create_task(None, kind, !T::SCOPED, T::INTERNAL);

        let kill = Rc::new(ObservableCell::new(false));
        let cell = kill.clone();

        let context = Context {
            hooks: self.hooks.clone(),
            id,
            tasks: self.manager.clone(),
            exception: self.exception.clone(),
            ctx: ctx.clone(),
            internal: T::INTERNAL,
        };

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
            hooks
                .borrow()
                .init(&ctx, id, kind, Some(manager.exectution_trigger_id()))?;
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
                        *exception.borrow_mut() = Some(err.into());
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

            manager.destroy_task(id, &ctx, &hooks, true).ok();
        });

        Ok(TaskHandle { id, kind, cell })
    }

    pub fn snapshot(&self, ctx: &Ctx<'js>) -> rquickjs::Result<Class<'js, Snapshot<'js>>> {
        let current = self.manager.exectution_trigger_id();

        if let Some(task) = self.manager.0.borrow_mut().tasks.get_mut(&current) {
            let context = Context {
                id: current,
                tasks: self.manager.clone(),
                hooks: self.hooks.clone(),
                exception: self.exception.clone(),
                internal: task.internal,
                ctx: ctx.clone(),
            };

            let snapshot = Class::instance(ctx.clone(), Snapshot { context })?;

            Runtime::from_ctx(ctx)?.borrow().finalizers.register(
                snapshot.clone().into_value(),
                current.into_js(ctx)?,
                None,
            )?;

            task.references += 1;

            Ok(snapshot)
        } else {
            throw!(ctx, "could not find current async task")
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

#[rquickjs::class(crate = "rquickjs")]
pub struct Snapshot<'js> {
    context: Context<'js>,
}

impl<'js> Trace<'js> for Snapshot<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.context.trace(tracer);
    }
}

unsafe impl<'js> JsLifetime<'js> for Snapshot<'js> {
    type Changed<'to> = Snapshot<'to>;
}

impl<'js> Snapshot<'js> {
    pub fn run(
        &self,
        ctx: Ctx<'js>,
        cb: Function<'js>,
        rest: Rest<Value<'js>>,
    ) -> rquickjs::Result<Value<'js>> {
        let mut args = Args::new(ctx.clone(), rest.len());
        args.push_args(rest.0)?;
        self.context.invoke_callback_arg(cb, args)
    }
}
