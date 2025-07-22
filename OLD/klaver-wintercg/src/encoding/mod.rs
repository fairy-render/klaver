mod b64;
mod encoding;

use rquickjs::prelude::Func;

pub use self::{b64::*, encoding::*};

pub fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
    decl.declare(stringify!(TextDecoder))?;
    decl.declare(stringify!(TextEncoder))?;
    decl.declare(stringify!(atob))?;
    decl.declare(stringify!(btoa))?;
    Ok(())
}

pub fn register<'js>(ctx: &rquickjs::prelude::Ctx<'js>) -> rquickjs::Result<()> {
    define!(ctx, TextEncoder, TextDecoder);
    ctx.globals().set(stringify!(atob), Func::new(atob))?;
    ctx.globals().set(stringify!(btoa), Func::new(btoa))?;
    Ok(())
}
