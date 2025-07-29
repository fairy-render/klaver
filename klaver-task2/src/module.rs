use klaver_util::rquickjs::{self, Ctx, module::ModuleDef, prelude::Func};

use crate::{AsyncState, Hook, ScriptHook, exec_state::AsyncId, state::HookState};

pub struct TaskModule;

impl ModuleDef for TaskModule {
    fn declare<'js>(
        decl: &klaver_util::rquickjs::module::Declarations<'js>,
    ) -> klaver_util::rquickjs::Result<()> {
        decl.declare("triggerAsyncId")?;
        decl.declare("executionAsyncId")?;
        decl.declare("createHook")?;
        Ok(())
    }

    fn evaluate<'js>(
        ctx: &klaver_util::rquickjs::Ctx<'js>,
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

                let Some(id) = state.exec.exectution_trigger_id() else {
                    return Ok(AsyncId::root());
                };

                rquickjs::Result::Ok(id)
            }),
        )?;

        exports.export(
            "createHook",
            Func::new(|ctx: Ctx<'js>, hook: ScriptHook<'js>| {
                let listeners = HookState::get(&ctx)?;

                listeners
                    .borrow()
                    .hooks
                    .borrow_mut()
                    .add_listener(Hook::Script(hook));

                rquickjs::Result::Ok(())
            }),
        )?;

        Ok(())
    }
}
