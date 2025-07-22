mod controller;
mod from;
mod reader;
mod stream;
mod underlying_source;

use crate::ExportTarget;

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

pub fn export<'js, T: ExportTarget<'js>>(
    ctx: &rquickjs::Ctx<'js>,
    registry: &crate::Registry,
    exports: &T,
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
