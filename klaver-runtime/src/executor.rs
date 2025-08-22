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
        execution: Execution,
        runner: T,
    ) -> rquickjs::Result<R>
    where
        T: AsyncFnOnce(Context<'js>) -> rquickjs::Result<R>,
    {
        if !execution.kind.is_native() {
            throw!(@type ctx, "ResourceKind should be a nativekind");
        }

        let id = self
            .manager
            .create_task(None, execution.kind, execution.persist, false);
        let parent_id = self.manager.parent_id(id);

        self.hooks
            .borrow()
            .init(ctx, id, execution.kind, Some(parent_id))?;

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
              state.set(if execution.exit == ExitMode::Idle {
                TaskStatus::Idle
              } else {
                TaskStatus::Killed
              });
            }
            ret
          }
          _ = self.exception.subscribe().fuse() => {
            todo!("Exception")
          }
        };

        if execution.wait {
            self.manager.wait_children(id).await;
        }

        self.manager.destroy_task(id, ctx, &self.hooks, true)?;

        if let Some(found) = self.exception.borrow().clone() {
            throw!(ctx, found);
        }

        ret
    }

    pub fn run<T, R>(&self, ctx: &Ctx<'js>, execution: Execution, runner: T) -> rquickjs::Result<R>
    where
        T: FnOnce(Context<'js>) -> rquickjs::Result<R>,
    {
        let id = self
            .manager
            .create_task(None, execution.kind, execution.persist, false);
        let parent_id = self.manager.parent_id(id);

        self.hooks
            .borrow()
            .init(ctx, id, execution.kind, Some(parent_id))?;

        let context = Context {
            hooks: self.hooks.clone(),
            id,
            tasks: self.manager.clone(),
            exception: self.exception.clone(),
            ctx: ctx.clone(),
            internal: false,
        };

        let ret = (runner)(context);

        if let Some(state) = self.manager.task_status(id) {
            state.set(if execution.exit == ExitMode::Idle {
                TaskStatus::Idle
            } else {
                TaskStatus::Killed
            });
        }

        if execution.wait {
            let manager = self.manager.clone();
            let hooks = self.hooks.clone();
            let cloned_ctx = ctx.clone();
            ctx.spawn(async move {
                manager.wait_children(id).await;
                manager.destroy_task(id, &cloned_ctx, &hooks, true).ok();
            });
        } else {
            self.manager.destroy_task(id, ctx, &self.hooks, true)?;
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

        let root_state = root_id.and_then(|root_id| self.manager.task_status(root_id));
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
                if T::SCOPED {
                    Either::Left(Either::Left(WaitIdle {
                        state: WaitIdleState::Init,
                        cell: ob.clone(),
                    }))
                } else {
                    Either::Left(Either::Right(WaitKilled {
                        state: WaitIdleState::Init,
                        cell: ob.clone(),
                    }))
                }
            } else {
                Either::Right(pending::<TaskStatus>())
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
                    // TaskStatus::Idle
                    TaskStatus::Killed
                }
                status = idle.fuse() => {

                    // TaskStatus::Idle
                    status
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

    pub fn snapshot(&self, ctx: &Ctx<'js>) -> rquickjs::Result<Class<'js, JsSnapshot<'js>>> {
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

            let snapshot = Class::instance(
                ctx.clone(),
                JsSnapshot {
                    context: Some(context),
                },
            )?;

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

    pub fn crate_task(&self, ctx: &Ctx<'js>) -> rquickjs::Result<Class<'js, JsSnapshot<'js>>> {
        let current = self
            .manager
            .create_task(None, ResourceKind::ROOT, true, false);

        self.hooks.borrow().init(
            ctx,
            current,
            ResourceKind::ROOT,
            Some(self.manager.exectution_trigger_id()),
        )?;

        if let Some(task) = self.manager.0.borrow_mut().tasks.get_mut(&current) {
            let context = Context {
                id: current,
                tasks: self.manager.clone(),
                hooks: self.hooks.clone(),
                exception: self.exception.clone(),
                internal: task.internal,
                ctx: ctx.clone(),
            };

            let snapshot = Class::instance(
                ctx.clone(),
                JsSnapshot {
                    context: Some(context),
                },
            )?;

            Runtime::from_ctx(ctx)?.borrow().finalizers.register(
                snapshot.clone().into_value(),
                current.into_js(ctx)?,
                None,
            )?;

            task.references += 1;

            let _ = task;

            self.manager
                .destroy_task(current, ctx, &self.hooks, false)?;

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
    type Output = TaskStatus;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        loop {
            let mut this = self.as_mut().project();

            let status = this.cell.get();

            if status == TaskStatus::Idle
                || status == TaskStatus::Failed
                || status == TaskStatus::Killed
            {
                return Poll::Ready(status);
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

pin_project! {
  struct WaitKilled {
      #[pin]
      state: WaitIdleState,
      cell: Rc<ObservableCell<TaskStatus>>,
  }
}

impl Future for WaitKilled {
    type Output = TaskStatus;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        loop {
            let mut this = self.as_mut().project();

            let status = this.cell.get();

            if status == TaskStatus::Failed || status == TaskStatus::Killed {
                return Poll::Ready(status);
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
pub struct JsSnapshot<'js> {
    context: Option<Context<'js>>,
}

impl<'js> Trace<'js> for JsSnapshot<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.context.trace(tracer);
    }
}

unsafe impl<'js> JsLifetime<'js> for JsSnapshot<'js> {
    type Changed<'to> = JsSnapshot<'to>;
}

impl<'js> JsSnapshot<'js> {
    pub fn run_callback(
        &self,
        ctx: Ctx<'js>,
        cb: Function<'js>,
        rest: Rest<Value<'js>>,
    ) -> rquickjs::Result<Value<'js>> {
        let Some(context) = self.context.as_ref() else {
            throw!(ctx, "Task is closed")
        };
        let mut args = Args::new(ctx.clone(), rest.len());
        args.push_args(rest.0)?;
        context.invoke_callback_arg(cb, args)
    }

    pub fn emit_destroy(&mut self) -> rquickjs::Result<()> {
        let Some(context) = self.context.take() else {
            return Ok(());
        };
        context
            .tasks
            .destroy_task(context.id, &context.ctx, &context.hooks, false)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitMode {
    Kill,
    Idle,
}

impl Default for ExitMode {
    fn default() -> Self {
        ExitMode::Idle
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Execution {
    pub exit: ExitMode,
    pub wait: bool,
    pub kind: ResourceKind,
    pub persist: bool,
}

impl Default for Execution {
    fn default() -> Self {
        Execution {
            exit: ExitMode::default(),
            wait: true,
            kind: ResourceKind::ROOT,
            persist: false,
        }
    }
}

impl Execution {
    pub fn exit(mut self, mode: ExitMode) -> Self {
        self.exit = mode;
        self
    }

    pub fn wait(mut self, wait: bool) -> Self {
        self.wait = wait;
        self
    }

    pub fn kind(mut self, kind: ResourceKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn persist(mut self, persist: bool) -> Self {
        self.persist = persist;
        self
    }
}
