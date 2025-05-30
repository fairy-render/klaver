mod datetime;
mod locale;
mod numberformat;
pub mod provider;

#[cfg(feature = "icu-compiled")]
pub mod baked;

use locale::JsLocale;
use numberformat::NumberFormat;
use rquickjs::{Class, Object};

pub use self::datetime::DateTimeFormat;

pub fn register<'js>(ctx: &rquickjs::prelude::Ctx<'js>) -> rquickjs::Result<()> {
    let intl = Object::new(ctx.clone())?;

    Class::<DateTimeFormat>::define(&intl)?;
    Class::<NumberFormat>::define(&intl)?;
    Class::<JsLocale>::define(&intl)?;

    ctx.globals().set("Intl", intl)?;

    Ok(())
}
