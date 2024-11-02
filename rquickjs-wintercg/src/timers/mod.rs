mod state;

use crate::config::WinterCG;
use rquickjs::{
    prelude::{Func, Opt},
    AsyncContext, Class, Function, IntoJs,
};
use rquickjs_util::util::FunctionExt;
pub use state::*;

#[rquickjs::function]
fn set_timeout<'js>(
    winter: Class<'js, WinterCG<'js>>,
    repeat: bool,
    func: Function<'js>,
    timeout: Opt<u64>,
) -> rquickjs::Result<TimeId> {
    winter
        .borrow()
        .timers()
        .create_timer(func, timeout.unwrap_or_default(), repeat)
}

#[rquickjs::function]
fn clear_timeout<'js>(winter: Class<'js, WinterCG<'js>>, timer: TimeId) -> rquickjs::Result<()> {
    winter.borrow().timers().clear_timer(timer)?;
    Ok(())
}

pub fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
    decl.declare(stringify!(setTimeout))?;
    decl.declare(stringify!(setInterval))?;
    decl.declare(stringify!(clearTimeout))?;
    decl.declare(stringify!(clearInterval))?;

    Ok(())
}

pub fn evaluate<'js>(
    ctx: &rquickjs::prelude::Ctx<'js>,
    exports: &rquickjs::module::Exports<'js>,
    winter: &Class<'js, WinterCG<'js>>,
) -> rquickjs::Result<()> {
    let timer = Func::new(set_timeout).into_js(ctx)?.get::<Function>()?;
    let clear = Func::new(clear_timeout).into_js(ctx)?.get::<Function>()?;

    exports.export(
        "setTimeout",
        timer.bind(ctx.clone(), (ctx.globals(), winter.clone(), false)),
    )?;

    exports.export("clearTimeout", clear.clone())?;

    exports.export(
        "setInterval",
        timer.bind(ctx.clone(), (ctx.globals(), winter.clone(), true)),
    )?;

    exports.export("clearInterval", clear.clone())?;

    Ok(())
}

pub async fn wait_timers<'a>(context: &'a AsyncContext) -> rquickjs::Result<()> {
    loop {
        let has_timers = rquickjs::async_with!(context => |ctx| {
          process_timers(&ctx).await
        })
        .await?;

        if !has_timers && !context.runtime().is_job_pending().await {
            break;
        }

        let sleep = rquickjs::async_with!(context => |ctx| {
          poll_timers(&ctx).await
        })
        .await?;

        sleep.await;
    }

    Ok(())
}
