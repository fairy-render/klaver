mod b64;
mod encoding;

use crate::ExportTarget;

pub use self::{
    b64::{atob, btoa},
    encoding::{TextDecoder, TextEncoder},
};
use rquickjs::{class::JsClass, prelude::Func};

pub fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
    declare!(decl, TextDecoder, TextEncoder);

    decl.declare("atob")?;
    decl.declare("btoa")?;

    Ok(())
}

pub fn export<'js, T>(
    ctx: &rquickjs::Ctx<'js>,
    registry: &crate::Registry,
    exports: &T,
) -> rquickjs::Result<()>
where
    T: ExportTarget<'js>,
{
    export!(ctx, registry, exports, TextDecoder, TextEncoder);

    exports.set(ctx, "atob", Func::new(atob))?;
    exports.set(ctx, "btoa", Func::new(btoa))?;

    Ok(())
}
