mod b64;
mod encoding;

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

pub fn export<'js>(
    ctx: &rquickjs::Ctx<'js>,
    registry: &crate::Registry,
    exports: &rquickjs::module::Exports<'js>,
) -> rquickjs::Result<()> {
    export!(ctx, registry, exports, TextDecoder, TextEncoder);

    exports.export("atob", Func::new(atob))?;
    exports.export("btoa", Func::new(btoa))?;

    Ok(())
}
