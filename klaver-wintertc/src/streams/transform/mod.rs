mod controller;
mod stream;
mod transformer;

use rquickjs::class::JsClass;

pub use self::{controller::TransformStreamDefaultController, stream::TransformStream};

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
