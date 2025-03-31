mod state;
mod timer;

use crate::config::WinterCG;
use rquickjs::{
    prelude::{Func, Opt},
    AsyncContext, Class, Function, IntoJs,
};
use rquickjs_util::{util::FunctionExt, RuntimeError};
pub use state::*;

#[rquickjs::function]
fn set_timeout<'js>(
    winter: Class<'js, WinterCG<'js>>,
    repeat: bool,
    func: Function<'js>,
    timeout: Opt<u64>,
) -> rquickjs::Result<TimeId> {
    winter.borrow().timers().create_timer(
        func.ctx().clone(),
        func,
        timeout.unwrap_or_default(),
        repeat,
    )
}

#[rquickjs::function]
fn clear_timeout<'js>(winter: Class<'js, WinterCG<'js>>, timer: TimeId) -> rquickjs::Result<()> {
    winter.borrow().timers().clear_timer(timer)?;
    Ok(())
}

pub fn register<'js>(
    ctx: &rquickjs::prelude::Ctx<'js>,
    winter: &Class<'js, WinterCG<'js>>,
) -> rquickjs::Result<()> {
    let timer = Func::new(set_timeout).into_js(ctx)?.get::<Function>()?;
    let clear = Func::new(clear_timeout)
        .into_js(ctx)?
        .get::<Function>()?
        .bind(ctx.clone(), (ctx.globals(), winter.clone()))?;

    let globals = ctx.globals();

    globals.set(
        "setTimeout",
        timer.bind(ctx.clone(), (ctx.globals(), winter.clone(), false)),
    )?;

    globals.set("clearTimeout", clear.clone())?;

    globals.set(
        "setInterval",
        timer.bind(ctx.clone(), (ctx.globals(), winter.clone(), true)),
    )?;

    globals.set("clearInterval", clear.clone())?;

    Ok(())
}

pub async fn wait_timers<'a>(context: &'a AsyncContext) -> Result<(), rquickjs_util::RuntimeError> {
    let chan = match context
        .with(|ctx| {
            Result::<_, RuntimeError>::Ok(WinterCG::get(&ctx)?.borrow().timers().create_err_chan())
        })
        .await
    {
        Ok(ret) => ret,
        Err(err) => return Err(err),
    };

    if let Some(err) = chan.wait().await {
        return Err(err);
    }

    Ok(())
}
