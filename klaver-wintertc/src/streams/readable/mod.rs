mod byob_reader;
mod controller;
mod from;
mod queue;
mod reader;
mod resource;
mod source;
mod state;
mod stream;
mod tee;

use klaver_core::ExportTarget;

pub use self::{
    byob_reader::ReadableStreamBYOBReader,
    controller::ReadableStreamDefaultController,
    from::from,
    reader::ReadableStreamDefaultReader,
    source::{AsyncIteratorSource, IteratorSource, NativeSource, One, UnderlyingSource},
    stream::ReadableStream,
};

// Not part of the public WinterTC surface - used by `TransformStream` to push directly into a
// `ReadableStream`'s shared state, bypassing the normal `NativeSource`/`JsUnderlyingSource` pull
// model.
pub(crate) use self::state::ReadableStreamData;

use rquickjs::class::JsClass;

pub fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
    declare!(
        decl,
        ReadableStream,
        ReadableStreamDefaultController,
        ReadableStreamDefaultReader,
        ReadableStreamBYOBReader
    );
    Ok(())
}

pub fn export<'js, T: ExportTarget<'js>>(
    ctx: &rquickjs::Ctx<'js>,
    registry: &klaver_core::Registry,
    exports: &T,
) -> rquickjs::Result<()> {
    export!(
        ctx,
        registry,
        exports,
        ReadableStream,
        ReadableStreamDefaultController,
        ReadableStreamDefaultReader,
        ReadableStreamBYOBReader
    );
    Ok(())
}
