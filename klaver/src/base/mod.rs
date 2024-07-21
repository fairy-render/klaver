use klaver_shared::format;
use rquickjs::{
    function::{Func, Opt, Rest},
    Ctx, Value,
};

pub use self::core::{get_core, Core, Extensions};

mod core;
pub mod timers;

pub fn init<'js>(ctx: Ctx<'js>) -> rquickjs::Result<()> {
    ctx.globals().set("Core", Core::new(ctx.clone())?)?;

    ctx.globals().set(
        "print",
        Func::new(|ctx: Ctx<'js>, arg: Rest<Value<'js>>| {
            let arg = if arg.len() == 1 {
                format(ctx, arg[0].clone(), None)?
            } else {
                let str = arg
                    .0
                    .into_iter()
                    .map(move |m| format(ctx.clone(), m, None))
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
