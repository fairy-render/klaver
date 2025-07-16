use rquickjs::{Class, class::JsClass, module::ModuleDef};

use crate::timers::Timers;

#[derive(Default)]
pub struct TimeModule;

impl ModuleDef for TimeModule {
    fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
        decl.declare(Timers::NAME)?;
        Ok(())
    }

    fn evaluate<'js>(
        ctx: &rquickjs::Ctx<'js>,
        exports: &rquickjs::module::Exports<'js>,
    ) -> rquickjs::Result<()> {
        exports.export(Timers::NAME, Class::<Timers>::create_constructor(ctx)?)?;
        Ok(())
    }
}
