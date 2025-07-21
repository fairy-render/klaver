mod controller;
mod from;
mod reader;
mod stream;
mod underlying_source;

pub use self::{
    controller::ReadableStreamDefaultController, reader::ReadableStreamDefaultReader,
    stream::ReadableStream, underlying_source::*,
};

use rquickjs::class::JsClass;

pub fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
    declare!(
        decl,
        ReadableStream,
        ReadableStreamDefaultController,
        ReadableStreamDefaultReader
    );
    Ok(())
}

pub fn export<'js>(
    ctx: &rquickjs::Ctx<'js>,
    registry: &crate::Registry,
    exports: &rquickjs::module::Exports<'js>,
) -> rquickjs::Result<()> {
    export!(
        ctx,
        registry,
        exports,
        ReadableStream,
        ReadableStreamDefaultController,
        ReadableStreamDefaultReader
    );
    Ok(())
}
