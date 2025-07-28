use klaver_util::{
    rquickjs::{self, Ctx, module::ModuleDef, prelude::Func},
    throw,
};

use crate::AsyncState;

pub struct TaskModule;

impl ModuleDef for TaskModule {
    fn declare<'js>(
        decl: &klaver_util::rquickjs::module::Declarations<'js>,
    ) -> klaver_util::rquickjs::Result<()> {
        decl.declare("triggerAsyncId")?;
        decl.declare("executionAsyncId")?;
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

                let Some(id) = state.exec.trigger_async_id() else {
                    throw!(@internal ctx, "No trigger id")
                };

                rquickjs::Result::Ok(id)
            }),
        )?;

        exports.export(
            "executionAsyncId",
            Func::new(|ctx: Ctx<'js>| {
                let state = AsyncState::get(&ctx)?;

                let Some(id) = state.exec.exectution_trigger_id() else {
                    throw!(@internal ctx, "No execution id")
                };

                rquickjs::Result::Ok(id)
            }),
        )?;

        Ok(())
    }
}
