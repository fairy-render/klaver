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

pub fn register<'js>(ctx: &rquickjs::prelude::Ctx<'js>) -> rquickjs::Result<()> {
    define!(
        ctx,
        ReadableStream,
        ReadableStreamDefaultReader,
        CountQueuingStrategy,
        ByteLengthQueuingStrategy
    );
    ReadableStream::add_async_iterable_prototype(ctx)?;

    Ok(())
}
