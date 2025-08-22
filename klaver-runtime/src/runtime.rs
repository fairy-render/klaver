use std::{cell::RefCell, rc::Rc};

use klaver_util::{
    CaugthException, FinalizationRegistry,
    rquickjs::{self, Class, Ctx, Function, IntoJs, JsLifetime, class::Trace, prelude::Func},
    sync::ObservableRefCell,
};

use crate::{
    id::AsyncId,
    listener::{HandleMap, HookListeners},
    resource::ResourceMap,
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

                state
                    .borrow_mut()
                    .manager
                    .destroy_task(id, &ctx, &hooks, false)?;

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
