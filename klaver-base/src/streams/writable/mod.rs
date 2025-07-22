mod controller;
// mod state;
mod stream;
mod underlying_sink;
mod writer;

use rquickjs::class::JsClass;

use crate::{ExportTarget, Registry};

pub use self::{
    controller::WritableStreamDefaultController, stream::WritableStream,
    writer::WritableStreamDefaultWriter,
};

pub fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
    declare!(
        decl,
        WritableStream,
        WritableStreamDefaultController,
        WritableStreamDefaultWriter
    );
    Ok(())
}

pub fn export<'js, T: ExportTarget<'js>>(
    ctx: &rquickjs::Ctx<'js>,
    registry: &Registry,
    exports: &T,
) -> rquickjs::Result<()> {
    export!(
        ctx,
        registry,
        exports,
        WritableStream,
        WritableStreamDefaultController,
        WritableStreamDefaultWriter
    );
    Ok(())
}
