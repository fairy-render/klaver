use rquickjs::{Ctx, Object};

pub fn process<'js>(ctx: Ctx<'js>) -> rquickjs::Result<Object<'js>> {
    let obj = Object::new(ctx.clone())?;

    let env = Object::new(ctx.clone())?;

    obj.set("env", env)?;

    Ok(obj)
}
