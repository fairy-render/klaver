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

pub fn evaluate<'js>(
    ctx: &rquickjs::prelude::Ctx<'js>,
    exports: &rquickjs::module::Exports<'js>,
) -> rquickjs::Result<()> {
    export!(exports, ctx, TextEncoder, TextDecoder);
    exports.export(stringify!(atob), Func::new(atob))?;
    exports.export(stringify!(btoa), Func::new(btoa))?;
    Ok(())
}
