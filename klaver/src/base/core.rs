use rquickjs::{Ctx, Object};

const CORE_KEY: &str = "Klaver";

pub fn get_core<'js>(ctx: &Ctx<'js>) -> rquickjs::Result<Object<'js>> {
    if let Ok(core) = ctx.globals().get(CORE_KEY) {
        Ok(core)
    } else {
        let o = Object::new(ctx.clone())?;
        ctx.globals().set(CORE_KEY, o.clone())?;
        Ok(o)
    }
}
