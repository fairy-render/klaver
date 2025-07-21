mod controller;
// mod state;
mod stream;
mod underlying_sink;
mod writer;

use rquickjs::class::JsClass;

use crate::Registry;

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

pub fn export<'js>(
    ctx: &rquickjs::Ctx<'js>,
    registry: &Registry,
    exports: &rquickjs::module::Exports<'js>,
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
