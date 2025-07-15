mod controller;
mod state;
mod stream;
mod underlying_sink;
mod writer;

use rquickjs::class::JsClass;

pub use self::{
    controller::WritableStreamDefaultController, stream::WritableStream, underlying_sink::*,
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

pub fn evaluate<'js>(
    ctx: &rquickjs::Ctx<'js>,
    exports: &rquickjs::module::Exports<'js>,
) -> rquickjs::Result<()> {
    define!(
        exports,
        ctx,
        WritableStream,
        WritableStreamDefaultController,
        WritableStreamDefaultWriter
    );
    Ok(())
}
