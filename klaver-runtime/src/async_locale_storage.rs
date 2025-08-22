use klaver_util::rquickjs::{self, Class, Ctx, Function, JsLifetime, Value, class::Trace};

use crate::{ResourceKind, executor::TaskExecutor, runtime::Runtime};

#[rquickjs::class(crate = "rquickjs")]
pub struct AsyncLocalStorage<'js> {
    runtime: TaskExecutor<'js>,
}

impl<'js> Trace<'js> for AsyncLocalStorage<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.runtime.trace(tracer);
    }
}

unsafe impl<'js> JsLifetime<'js> for AsyncLocalStorage<'js> {
    type Changed<'to> = AsyncLocalStorage<'to>;
}

#[rquickjs::methods(crate = "rquickjs")]
impl<'js> AsyncLocalStorage<'js> {
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'js>) -> rquickjs::Result<AsyncLocalStorage<'js>> {
        Ok(AsyncLocalStorage {
            runtime: TaskExecutor::from_ctx(&ctx)?,
        })
    }
    pub fn run(
        &self,
        ctx: Ctx<'js>,
        store: Value<'js>,
        cb: Function<'js>,
    ) -> rquickjs::Result<Value<'js>> {
        self.runtime
            .run(&ctx, ResourceKind::STORAGE, move |context| {
                context.handle()?.set("store", store)?;
                context.invoke_callback(cb, ())
            })
    }

    #[qjs(rename = "getStore")]
    pub fn get_store(&self, ctx: Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        let current = self.runtime.manager().exectution_trigger_id();
        // println!("{} {:#?}", current, self.runtime.manager());
        if let Some(id) = self
            .runtime
            .manager()
            .find_parent(current, |task| task.kind == ResourceKind::STORAGE)
        {
            match self.runtime.hooks().borrow().get_resource_handle(&ctx, id) {
                Ok(ret) => ret.get("store"),
                _ => Ok(Value::new_null(ctx)),
            }
        } else {
            Ok(Value::new_null(ctx))
        }
    }
}
