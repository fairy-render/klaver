use crate::{
    ResourceKind, async_hook::AsyncHook, async_locale_storage::AsyncLocalStorage,
    async_resource::AsyncResource, listener::ScriptListener, runtime::Runtime,
};
use klaver_util::rquickjs::{
    self, Class, Ctx, String, Value, class::JsClass, module::ModuleDef, prelude::Func,
};

pub struct TaskModule;

impl ModuleDef for TaskModule {
    fn declare<'js>(
        decl: &klaver_util::rquickjs::module::Declarations<'js>,
    ) -> klaver_util::rquickjs::Result<()> {
        decl.declare("triggerAsyncId")?;
        decl.declare("executionAsyncId")?;
        decl.declare("createHook")?;
        decl.declare("executionAsyncResource")?;
        decl.declare("resourceName")?;
        decl.declare(AsyncLocalStorage::NAME)?;
        decl.declare(AsyncResource::NAME)?;
        Ok(())
    }

    fn evaluate<'js>(
        ctx: &klaver_util::rquickjs::Ctx<'js>,
        exports: &klaver_util::rquickjs::module::Exports<'js>,
    ) -> klaver_util::rquickjs::Result<()> {
        exports.export(
            "triggerAsyncId",
            Func::new(|ctx: Ctx<'js>| {
                let state = Runtime::from_ctx(&ctx)?;

                rquickjs::Result::Ok(state.borrow().manager.trigger_async_id())
            }),
        )?;

        exports.export(
            "executionAsyncId",
            Func::new(|ctx: Ctx<'js>| {
                let state = Runtime::from_ctx(&ctx)?;

                rquickjs::Result::Ok(state.borrow().manager.exectution_trigger_id())
            }),
        )?;

        exports.export(
            "createHook",
            Func::new(|ctx: Ctx<'js>, hook: ScriptListener<'js>| {
                let state = Runtime::from_ctx(&ctx)?;
                let runtime = state.borrow();
                let hook = AsyncHook::new(hook, runtime.hooks.clone());

                rquickjs::Result::Ok(hook)
            }),
        )?;

        exports.export(
            "executionAsyncResource",
            Func::new(|ctx: Ctx<'js>| {
                let state = Runtime::from_ctx(&ctx)?;
                let runtime = state.borrow();

                let resource = runtime
                    .hooks
                    .borrow()
                    .get_resource_handle(&ctx, runtime.manager.exectution_trigger_id())?;

                rquickjs::Result::Ok(resource)
            }),
        )?;

        exports.export(
            "resourceName",
            Func::new(|ctx: Ctx<'js>, resource: ResourceKind| {
                let state = Runtime::from_ctx(&ctx)?;

                if let Some(name) = state.borrow().resource_map.borrow().name(resource) {
                    rquickjs::Result::Ok(String::from_str(ctx, name)?.into_value())
                } else {
                    Ok(Value::new_null(ctx))
                }
            }),
        )?;

        exports.export(
            AsyncLocalStorage::NAME,
            Class::<AsyncLocalStorage>::create_constructor(ctx)?,
        )?;

        exports.export(
            AsyncResource::NAME,
            Class::<AsyncResource>::create_constructor(ctx)?,
        )?;

        Ok(())
    }
}

#[cfg(feature = "module")]
klaver_modules::module_info!("node:async_hooks" => TaskModule);
