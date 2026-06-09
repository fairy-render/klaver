use klaver_core::{Exportable, FunctionExt};
use rquickjs::{
    Class, Ctx, Function, IntoJs,
    class::JsClass,
    module::ModuleDef,
    prelude::{Func, Opt},
};

use super::{TimeId, timers::Timers};

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

impl<'js> Exportable<'js> for TimeModule {
    fn export<T>(
        ctx: &rquickjs::Ctx<'js>,
        _registry: &klaver_core::Registry,
        target: &T,
    ) -> rquickjs::Result<()>
    where
        T: klaver_core::ExportTarget<'js>,
    {
        let timers = Timers::new(ctx.clone())?;

        let set_timeout = Func::new(
            |ctx: Ctx<'js>,
             timers: Class<'js, Timers>,
             repeat: bool,
             callback: Function<'js>,
             timeout: Opt<u64>| {
                timers
                    .borrow_mut()
                    .create_timeout(ctx, callback, timeout, Opt(Some(repeat)))
            },
        )
        .into_js(&ctx)?
        .get::<Function>()?;

        let globals = ctx.globals();

        let clear_timeout = Func::new(|timers: Class<'js, Timers>, time: TimeId| {
            timers.borrow_mut().clear_timeout(time)
        })
        .into_js(&ctx)?
        .get::<Function>()?
        .bind(&ctx, (globals.clone(), timers.clone()))?;

        target.set(
            ctx,
            "setTimeout",
            set_timeout.bind(&ctx, (globals.clone(), timers.clone(), false))?,
        )?;

        target.set(
            ctx,
            "setInterval",
            set_timeout.bind(&ctx, (globals.clone(), timers.clone(), true))?,
        )?;

        target.set(ctx, "clearTimeout", clear_timeout.clone())?;
        target.set(ctx, "clearInterval", clear_timeout.clone())?;

        Ok(())
    }
}

#[cfg(feature = "module")]
impl klaver_modules::Global for TimeModule {
    async fn define<'a, 'js: 'a>(&'a self, ctx: rquickjs::Ctx<'js>) -> rquickjs::Result<()> {
        Self::export(
            &ctx,
            &klaver_core::Registry::instance(&ctx)?,
            &ctx.globals(),
        )?;
        Ok(())
    }
}

#[cfg(feature = "module")]
impl klaver_modules::GlobalInfo for TimeModule {
    fn register(builder: &mut klaver_modules::GlobalBuilder<'_, Self>) {
        builder.register(Self);
    }
}
