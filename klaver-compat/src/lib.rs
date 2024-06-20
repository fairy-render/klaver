use rquickjs::{Ctx, Promise};

const SOURCE: &[u8] = include_bytes!("compat.js");

fn load<'js>(ctx: &Ctx<'js>) -> rquickjs::Result<Promise<'js>> {
    klaver_base::register_global(ctx)?;
    ctx.eval_promise(SOURCE)
}

pub fn init(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
    load(ctx)?.finish()
}

pub async fn init_async(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
    load(ctx)?.into_future().await
}

// pub fn register_global(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
//     Class::<klaver_base::module::TextDecoder>::register(ctx)?;
//     Class::<klaver_base::module::TextDecoder>::define(&ctx.globals())?;

//     ctx.globals().set(
//         "setTimeout",
//         Function::new(ctx.clone(), klaver_base::module::base_mod::set_timeout),
//     )?;

//     ctx.globals().set(
//         "setInterval",
//         Function::new(ctx.clone(), klaver_base::module::base_mod::set_interval),
//     )?;

//     ctx.globals().set(
//         "clearTimeout",
//         Function::new(ctx.clone(), klaver_base::module::base_mod::clear_timeout),
//     )?;

//     ctx.globals().set(
//         "clearInterval",
//         Function::new(ctx.clone(), klaver_base::module::base_mod::clear_timeout),
//     )?;

//     Ok(())
// }
