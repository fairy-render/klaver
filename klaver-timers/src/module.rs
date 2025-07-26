use klaver_util::FunctionExt;
use rquickjs::{
    Class, Ctx, Function, IntoJs,
    class::JsClass,
    module::ModuleDef,
    prelude::{Func, Opt},
};

use crate::{TimeId, timers::Timers};

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

#[cfg(feature = "module")]
impl klaver_modules::Global for TimeModule {
    async fn define<'a, 'js: 'a>(&'a self, ctx: rquickjs::Ctx<'js>) -> rquickjs::Result<()> {
        let timers = Timers::new(ctx.clone())?;
        let globals = ctx.globals();

        let set_timeout = Func::new(
            |ctx: Ctx<'js>,
             timers: Class<'js, Timers<'js>>,
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

        let clear_timeout = Func::new(|timers: Class<'js, Timers<'js>>, time: TimeId| {
            timers.borrow_mut().clear_timeout(time)
        })
        .into_js(&ctx)?
        .get::<Function>()?
        .bind(&ctx, (globals.clone(), timers.clone()))?;

        globals.set(
            "setTimeout",
            set_timeout.bind(&ctx, (globals.clone(), timers.clone(), false))?,
        )?;

        globals.set(
            "setInterval",
            set_timeout.bind(&ctx, (globals.clone(), timers.clone(), true))?,
        )?;

        globals.set("clearTimeout", clear_timeout.clone())?;
        globals.set("clearInterval", clear_timeout.clone())?;

        Ok(())
    }
}

#[cfg(feature = "module")]
impl klaver_modules::GlobalInfo for TimeModule {
    fn register(builder: &mut klaver_modules::GlobalBuilder<'_, Self>) {
        builder.register(Self);
    }
}
