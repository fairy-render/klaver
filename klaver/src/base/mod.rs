use rquickjs::{function::Func, Ctx};

mod core;
mod encoding;
pub mod timers;

pub fn init<'js>(ctx: Ctx<'js>) -> rquickjs::Result<()> {
    encoding::init(&ctx)?;
    timers::init(&ctx)?;

    ctx.globals().set(
        "print",
        Func::new(|arg: String| {
            println!("{arg}");
            rquickjs::Result::Ok(())
        }),
    )?;

    Ok(())
}
