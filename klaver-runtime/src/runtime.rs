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
    CaugthException, FinalizationRegistry,
    rquickjs::{
        self, CatchResultExt, Class, Ctx, Function, IntoJs, JsLifetime, Value, class::Trace,
        prelude::Func,
    },
    sync::{ObservableCell, ObservableRefCell},
};
use pin_project_lite::pin_project;

use crate::{
    context::Context,
    id::AsyncId,
    listener::{HandleMap, HookListeners},
    resource::{Resource, ResourceKind, ResourceMap},
    task::TaskStatus,
    task_manager::TaskManager,
};

#[rquickjs::class(crate = "rquickjs")]
pub(crate) struct Runtime<'js> {
    pub hooks: Class<'js, HookListeners<'js>>,
    pub manager: TaskManager,
    pub resource_map: Rc<RefCell<ResourceMap>>,
    pub exception: Rc<ObservableRefCell<Option<CaugthException>>>,
    pub finalizers: FinalizationRegistry<'js>,
}

unsafe impl<'js> JsLifetime<'js> for Runtime<'js> {
    type Changed<'to> = Runtime<'to>;
}

impl<'js> Trace<'js> for Runtime<'js> {
    fn trace<'a>(&self, tracer: klaver_util::rquickjs::class::Tracer<'a, 'js>) {
        self.hooks.trace(tracer);
        self.finalizers.trace(tracer);
    }
}

impl<'js> Runtime<'js> {
    pub fn from_ctx(ctx: &Ctx<'js>) -> rquickjs::Result<Class<'js, Runtime<'js>>> {
        if let Ok(runtime) = ctx.globals().get("$__runtime") {
            Ok(runtime)
        } else {
            let hooks = Class::instance(ctx.clone(), HookListeners::new(HandleMap::new(ctx)?)?)?;
            let manager = TaskManager::default();
            let resource_map = ResourceMap::new();
            let exception = Rc::new(ObservableRefCell::new(None));

            let handler = Func::new(|ctx: Ctx<'js>, id: AsyncId| {
                let state = Runtime::from_ctx(&ctx)?;

                let hooks = state.borrow().hooks.clone();

                state.borrow_mut().manager.destroy_task(id, &ctx, &hooks)?;

                rquickjs::Result::Ok(())
            })
            .into_js(&ctx)?
            .get::<Function<'js>>()?;

            let finalizers = FinalizationRegistry::new(ctx.clone(), handler)?;

            let output = Class::instance(
                ctx.clone(),
                Runtime {
                    hooks,
                    manager,
                    resource_map: Rc::new(RefCell::new(resource_map)),
                    exception,
                    finalizers,
                },
            )?;

            ctx.globals().set("$__runtime", output.clone())?;

            Ok(output)
        }
    }
}

// pin_project! {
//     #[project = WaitIdleStateProj]
//     enum WaitIdleState {
//         Init,
//         Waiting {
//             #[pin]
//             future: klaver_util::sync::Listener
//         }
//     }
// }

// pin_project! {
//     struct WaitIdle {
//         #[pin]
//         state: WaitIdleState,
//         cell: Rc<ObservableCell<TaskStatus>>,
//     }
// }

// impl Future for WaitIdle {
//     type Output = ();

//     fn poll(
//         mut self: std::pin::Pin<&mut Self>,
//         cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<Self::Output> {
//         loop {
//             let mut this = self.as_mut().project();

//             if this.cell.get() == TaskStatus::Idle {
//                 return Poll::Ready(());
//             }

//             match this.state.as_mut().project() {
//                 WaitIdleStateProj::Init => {
//                     let future = this.cell.subscribe();
//                     this.state.set(WaitIdleState::Waiting { future });
//                 }
//                 WaitIdleStateProj::Waiting { future } => {
//                     ready!(future.poll(cx));
//                     this.state.set(WaitIdleState::Init);
//                 }
//             }
//         }
//     }
// }
