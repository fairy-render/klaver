use klaver_util::{AsyncIter, Buffer, Iter, is_async_iterable, is_iteratable};
use rquickjs::{Class, Ctx, FromJs, Value};

use crate::streams::readable::{AsyncIteratorSource, IteratorSource, One};

use super::ReadableStream;

pub fn from<'js>(
    ctx: Ctx<'js>,
    value: Value<'js>,
) -> rquickjs::Result<Class<'js, ReadableStream<'js>>> {
    let read = if ReadableStream::is(&value) {
        Class::<ReadableStream>::from_js(&ctx, value)?
    } else if is_async_iterable(&ctx, &value) {
        let iter = value.get::<AsyncIter>()?;
        Class::instance(
            ctx.clone(),
            ReadableStream::from_native(ctx, AsyncIteratorSource(iter))?,
        )?
    } else if is_iteratable(&value) {
        let iter = value.get::<Iter>()?;
        Class::instance(
            ctx.clone(),
            ReadableStream::from_native(ctx, IteratorSource(iter))?,
        )?
    } else if Buffer::is(&ctx, &value)? {
        Class::instance(
            ctx.clone(),
            ReadableStream::from_native(ctx, One::new(value))?,
        )?
    } else {
        todo!()
    };

    Ok(read)
}
