mod datetime;
mod locale;

use locale::JsLocale;
use rquickjs::{Class, Object};

pub use self::datetime::DateTimeFormat;

pub fn register<'js>(ctx: &rquickjs::prelude::Ctx<'js>) -> rquickjs::Result<()> {
    let intl = Object::new(ctx.clone())?;

    Class::<DateTimeFormat>::define(&intl)?;
    Class::<JsLocale>::define(&intl)?;

    ctx.globals().set("Intl", intl)?;

    Ok(())
}
