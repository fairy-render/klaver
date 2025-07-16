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

pub fn evaluate<'js>(
    ctx: &rquickjs::Ctx<'js>,
    exports: &rquickjs::module::Exports<'js>,
) -> rquickjs::Result<()> {
    define!(
        exports,
        ctx,
        ReadableStream,
        ReadableStreamDefaultController,
        ReadableStreamDefaultReader
    );
    Ok(())
}
