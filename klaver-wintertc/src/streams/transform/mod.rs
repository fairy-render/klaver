mod controller;
mod stream;
mod transformer;

use rquickjs::class::JsClass;

pub use self::{controller::TransformStreamDefaultController, stream::TransformStream};

// Not part of the public WinterTC surface - reused by `TextEncoderStream`/`TextDecoderStream`,
// which are built the same way `TransformStream` is: a passive readable side fed directly by the
// writable side's native sink via the shared `TransformStreamDefaultController`.
pub(crate) use self::stream::PassiveSource;

pub fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
    declare!(decl, TransformStream, TransformStreamDefaultController);
    Ok(())
}

pub fn export<'js, T: klaver_core::ExportTarget<'js>>(
    ctx: &rquickjs::Ctx<'js>,
    registry: &klaver_core::Registry,
    exports: &T,
) -> rquickjs::Result<()> {
    export!(
        ctx,
        registry,
        exports,
        TransformStream,
        TransformStreamDefaultController
    );
    Ok(())
}
