use klaver_util::FunctionExt;
use klaver_util::rquickjs::{
    self, Class, Ctx, Function, IntoJs, JsLifetime, Value, class::Trace, prelude::Func,
};

use crate::{
    AsyncState, NativeListener, ResourceKind, TaskCtx,
    listener::ResourceHandle,
    snapshot::{Snapshot, snapshot},
    state::HookState,
};

#[rquickjs::class(crate = "rquickjs")]
pub struct AsyncLocalStorage<'js> {
    state: AsyncState,

    hook: Class<'js, HookState<'js>>,
}

impl<'js> Trace<'js> for AsyncLocalStorage<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.hook.trace(tracer);
    }
}

unsafe impl<'js> JsLifetime<'js> for AsyncLocalStorage<'js> {
    type Changed<'to> = AsyncLocalStorage<'to>;
}

#[rquickjs::methods(crate = "rquickjs")]
impl<'js> AsyncLocalStorage<'js> {
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'js>) -> rquickjs::Result<AsyncLocalStorage<'js>> {
        let state = AsyncState::instance(&ctx)?;
        let hook = HookState::get(&ctx)?;

        Ok(AsyncLocalStorage { state, hook })
    }

    pub fn run(
        &self,
        ctx: Ctx<'js>,
        store: Value<'js>,
        func: Function<'js>,
    ) -> rquickjs::Result<Value<'js>> {
        let (ret, _) = self
            .state
            .invoke(&ctx, ResourceKind::STORAGE, false, move |ctx| {
                ctx.handle()?.set("store", store)?;
                ctx.invoke_callback(func, ())
            })?;

        Ok(ret)
    }

    #[qjs(rename = "getStore")]
    pub fn get_store(&self, ctx: Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        let mut id = self.state.exec.exectution_trigger_id();
        if let Some(store) = self
            .state
            .exec
            .find_parent(id, |task| task.kind == ResourceKind::STORAGE)
        {
            id = store;
        }

        let Ok(resource) = self.hook.borrow().resources.get_handle(&ctx, id) else {
            return Ok(Value::new_null(ctx));
        };
        resource.get("store")
    }

    #[qjs(static)]
    pub fn snapshot(ctx: Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        let state = AsyncState::instance(&ctx)?;

        let id = state.exec.exectution_trigger_id();

        let task_ctx = state.exec.task_ctx(&ctx, id)?;

        let snapshot = Class::instance(
            ctx.clone(),
            Snapshot {
                id,
                exec: Some(task_ctx),
            },
        )?;

        let func = Func::new(crate::snapshot::snapshot)
            .into_js(&ctx)?
            .get::<Function>()?
            .bind(&ctx, (ctx.globals(), snapshot))?;

        let value = func.into_value();

        let state = HookState::get(&ctx)?;

        state
            .borrow()
            .registry
            .register(value.clone(), id.into_js(&ctx)?, None)?;

        Ok(value)
    }
}
