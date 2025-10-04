mod controller;
mod from;
mod queue;
mod reader;
mod resource;
mod source;
mod state;
mod stream;

use crate::ExportTarget;

pub use self::{
    controller::ReadableStreamDefaultController,
    from::from,
    reader::ReadableStreamDefaultReader,
    source::{AsyncIteratorSource, IteratorSource, NativeSource, One, UnderlyingSource},
    stream::ReadableStream,
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
