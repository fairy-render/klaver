use rquickjs::{
    function::{Func, Opt, Rest},
    Class, Ctx, Value,
};

use self::core::Core;

mod core;
// mod encoding;
mod format;
pub mod timers;

pub fn init<'js>(ctx: Ctx<'js>) -> rquickjs::Result<()> {
    // encoding::init(&ctx)?;
    // timers::init(&ctx)?;

    ctx.globals().set("Core", Core::new(ctx.clone())?)?;

    ctx.globals().set(
        "print",
        Func::new(|ctx: Ctx<'js>, arg: Rest<Value<'js>>| {
            let arg = if arg.len() == 1 {
                format::format(ctx, arg[0].clone(), Opt(None))?
            } else {
                let str = arg
                    .0
                    .into_iter()
                    .map(move |m| format::format(ctx.clone(), m, Opt(None)))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(" ");
                str
            };
            println!("{arg}");
            rquickjs::Result::Ok(())
        }),
    )?;

    Ok(())
}
