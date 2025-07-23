use klaver_base::{Console, ExportTarget, Exportable, NullWriter, Registry};
use klaver_timers::{TimeId, Timers};
use rquickjs::{
    Class, Ctx, Function, IntoJs,
    prelude::{Func, Opt},
};
use rquickjs_util::util::FunctionExt;

pub fn export<'js, T: ExportTarget<'js>>(ctx: &Ctx<'js>, target: &T) -> rquickjs::Result<()> {
    let timers = Timers::new(ctx.clone())?;

    let set_timeout = Func::new(set_timeout)
        .into_js(ctx)?
        .into_function()
        .expect("set_timeout");

    let clear_timeout = Func::new(clear_timeout)
        .into_js(ctx)?
        .into_function()
        .expect("clear_timeout")
        .bind(ctx.clone(), (timers.clone(),))?;

    target.set(
        ctx,
        "setTimeout",
        set_timeout.bind(ctx.clone(), (ctx.globals(), timers.clone(), false)),
    )?;
    target.set(ctx, "clearTimeout", clear_timeout.clone())?;

    // target.set(
    //     ctx,
    //     "setInterval",
    //     set_interval.bind(ctx.clone(), (timers.clone(),)),
    // )?;
    // target.set(ctx, "clearInterval", clear_timeout)?;

    Ok(())
}

fn set_timeout<'js>(
    ctx: Ctx<'js>,
    timers: Class<'js, Timers<'js>>,
    repeat: bool,
    cb: Function<'js>,
    timeout: Opt<u64>,
) -> rquickjs::Result<TimeId> {
    timers
        .borrow_mut()
        .create_timeout(ctx.clone(), cb, timeout, Opt(Some(repeat)))
}

fn set_interval<'js>(
    ctx: Ctx<'js>,
    timers: Class<'js, Timers<'js>>,
    cb: Function<'js>,
    timeout: Opt<u64>,
) -> rquickjs::Result<TimeId> {
    timers
        .borrow_mut()
        .create_timeout(ctx.clone(), cb, timeout, Opt(Some(true)))
}

fn clear_timeout<'js>(timers: Class<'js, Timers<'js>>, id: TimeId) -> rquickjs::Result<()> {
    timers.borrow_mut().clear_timeout(id)
}
