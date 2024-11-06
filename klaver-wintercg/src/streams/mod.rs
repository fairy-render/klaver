mod readable_stream;

use rquickjs_util::async_iterator::AsyncIterable;

pub use self::readable_stream::*;

pub fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
    decl.declare(stringify!(ReadableStream))?;
    decl.declare(stringify!(ReadableStreamDefaultReader))?;
    decl.declare(stringify!(CountQueuingStrategy))?;
    decl.declare(stringify!(ByteLengthQueuingStrategy))?;
    Ok(())
}

pub fn evaluate<'js>(
    ctx: &rquickjs::prelude::Ctx<'js>,
    exports: &rquickjs::module::Exports<'js>,
) -> rquickjs::Result<()> {
    export!(
        exports,
        ctx,
        ReadableStream,
        ReadableStreamDefaultReader,
        CountQueuingStrategy,
        ByteLengthQueuingStrategy
    );
    ReadableStream::add_async_iterable_prototype(ctx)?;

    Ok(())
}
