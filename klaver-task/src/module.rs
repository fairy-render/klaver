use crate::{AsyncState, ResourceKind, ScriptListener, async_hook::AsyncHook, state::HookState};
use klaver_util::rquickjs::{self, Ctx, String, Value, module::ModuleDef, prelude::Func};

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
        Ok(())
    }

    fn evaluate<'js>(
        _ctx: &klaver_util::rquickjs::Ctx<'js>,
        exports: &klaver_util::rquickjs::module::Exports<'js>,
    ) -> klaver_util::rquickjs::Result<()> {
        exports.export(
            "triggerAsyncId",
            Func::new(|ctx: Ctx<'js>| {
                let state = AsyncState::get(&ctx)?;

                rquickjs::Result::Ok(state.exec.trigger_async_id())
            }),
        )?;

        exports.export(
            "executionAsyncId",
            Func::new(|ctx: Ctx<'js>| {
                let state = AsyncState::get(&ctx)?;

                rquickjs::Result::Ok(state.exec.exectution_trigger_id())
            }),
        )?;

        exports.export(
            "createHook",
            Func::new(|ctx: Ctx<'js>, hook: ScriptListener<'js>| {
                let listeners = HookState::get(&ctx)?.borrow().hooks.clone();

                let hook = AsyncHook::new(hook, listeners);

                rquickjs::Result::Ok(hook)
            }),
        )?;

        exports.export(
            "executionAsyncResource",
            Func::new(|ctx: Ctx<'js>| {
                let state = AsyncState::get(&ctx)?;
                let hooks = HookState::get(&ctx)?;

                let resource = hooks
                    .borrow()
                    .resources
                    .get_handle(&ctx, state.exec.exectution_trigger_id())?;

                rquickjs::Result::Ok(resource)
            }),
        )?;

        exports.export(
            "resourceName",
            Func::new(|ctx: Ctx<'js>, resource: ResourceKind| {
                let state = AsyncState::get(&ctx)?;

                if let Some(name) = state.resource_map.borrow().name(resource) {
                    rquickjs::Result::Ok(String::from_str(ctx, name)?.into_value())
                } else {
                    Ok(Value::new_null(ctx))
                }
            }),
        )?;

        Ok(())
    }
}
