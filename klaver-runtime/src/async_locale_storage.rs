use klaver_util::{
    FunctionExt,
    rquickjs::{
        self, Atom, Class, Ctx, Function, IntoJs, JsLifetime, String, Value,
        class::Trace,
        prelude::{Func, Rest},
        runtime,
    },
};

use crate::{
    AsyncId, Context, ResourceKind,
    executor::{Snapshot, TaskExecutor},
    runtime::Runtime,
};

#[rquickjs::class(crate = "rquickjs")]
pub struct AsyncLocalStorage<'js> {
    runtime: TaskExecutor<'js>,
    store_key: String<'js>,
}

impl<'js> Trace<'js> for AsyncLocalStorage<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.runtime.trace(tracer);
        self.store_key.trace(tracer);
    }
}

unsafe impl<'js> JsLifetime<'js> for AsyncLocalStorage<'js> {
    type Changed<'to> = AsyncLocalStorage<'to>;
}

#[rquickjs::methods(crate = "rquickjs")]
impl<'js> AsyncLocalStorage<'js> {
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'js>) -> rquickjs::Result<AsyncLocalStorage<'js>> {
        let store_key = String::from_str(ctx.clone(), "$AsyncLocaleStorageStoreKey")?;

        Ok(AsyncLocalStorage {
            runtime: TaskExecutor::from_ctx(&ctx)?,
            store_key,
        })
    }
    pub fn run(
        &self,
        ctx: Ctx<'js>,
        store: Value<'js>,
        cb: Function<'js>,
    ) -> rquickjs::Result<Value<'js>> {
        let key = self.store_key.clone();
        self.runtime
            .run(&ctx, ResourceKind::STORAGE, move |context| {
                context.handle()?.set(key, store)?;
                context.invoke_callback(cb, ())
            })
    }

    #[qjs(rename = "enterWith")]
    pub fn enter_with(&self, ctx: Ctx<'js>, store: Value<'js>) -> rquickjs::Result<()> {
        let current = self.runtime.manager().exectution_trigger_id();
        if let Some(id) = self
            .runtime
            .manager()
            .find_parent(current, |task| task.kind == ResourceKind::ROOT)
        {
            match self.runtime.hooks().borrow().get_resource_handle(&ctx, id) {
                Ok(ret) => {
                    ret.set(self.store_key.clone(), store)?;
                    Ok(())
                }
                _ => Ok(()),
            }
        } else {
            Ok(())
        }
    }

    #[qjs(rename = "getStore")]
    pub fn get_store(&self, ctx: Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        let current = self.runtime.manager().exectution_trigger_id();

        println!("Current {}", current);

        if let Some(id) = self
            .runtime
            .manager()
            .find_parent(current, |task| task.kind == ResourceKind::STORAGE)
        {
            match self.runtime.hooks().borrow().get_resource_handle(&ctx, id) {
                Ok(ret) => ret.get(self.store_key.clone()),
                _ => Ok(Value::new_null(ctx)),
            }
        } else if let Some(id) = self
            .runtime
            .manager()
            .find_parent(current, |task| task.kind == ResourceKind::ROOT)
        {
            match self.runtime.hooks().borrow().get_resource_handle(&ctx, id) {
                Ok(ret) => ret.get(self.store_key.clone()),
                _ => Ok(Value::new_null(ctx)),
            }
        } else {
            Ok(Value::new_null(ctx))
        }
    }

    #[qjs(static)]
    pub fn snapshot(ctx: Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        let executor = TaskExecutor::from_ctx(&ctx)?;
        let snapshot = executor.snapshot(&ctx)?;

        let func = Func::new(
            |ctx: Ctx<'js>,
             snapshot: Class<'js, Snapshot<'js>>,
             callback: Function<'js>,
             args: Rest<Value<'js>>| { snapshot.borrow().run(ctx, callback, args) },
        )
        .into_js(&ctx)?
        .get::<Function<'js>>()?
        .bind(&ctx, (ctx.globals(), snapshot))?;

        Ok(func.into_value())
    }
}
