use rquickjs::{Ctx, Promise};

const SOURCE: &[u8] = include_bytes!("compat.js");

fn load<'js>(ctx: &Ctx<'js>) -> rquickjs::Result<Promise<'js>> {
    // klaver_base::register_global(ctx)?;
    ctx.eval_promise(SOURCE)
}

pub fn init(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
    load(ctx)?.finish()
}

pub async fn init_async(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
    load(ctx)?.into_future().await
}
