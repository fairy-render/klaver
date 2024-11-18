mod dateformat;

use rquickjs::{Class, Object};

pub use self::dateformat::DateTimeFormat;

pub fn register<'js>(ctx: &rquickjs::prelude::Ctx<'js>) -> rquickjs::Result<()> {
    let intl = Object::new(ctx.clone())?;

    Class::<DateTimeFormat>::define(&intl)?;
    ctx.globals().set("Intl", intl)?;

    Ok(())
}
