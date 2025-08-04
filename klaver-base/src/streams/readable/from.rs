use klaver_util::{AsyncIterable, Buffer, Iterable, is_async_iterable, is_iteratable};
use rquickjs::{Class, Ctx, FromJs, Value};

use crate::streams::readable::{AsyncIteratorSource, IteratorSource, One};

use super::ReadableStream;

pub fn from<'js>(
    ctx: &Ctx<'js>,
    value: Value<'js>,
) -> rquickjs::Result<Class<'js, ReadableStream<'js>>> {
    let read = if ReadableStream::is(&value) {
        Class::<ReadableStream>::from_js(&ctx, value)?
    } else if is_async_iterable(&ctx, &value) {
        let iter = value.get::<AsyncIterable>()?;
        Class::instance(
            ctx.clone(),
            ReadableStream::from_native(ctx, AsyncIteratorSource(iter.async_iterator()?), None)?,
        )?
    } else if is_iteratable(&value) {
        let iter = value.get::<Iterable>()?.iterator()?;
        Class::instance(
            ctx.clone(),
            ReadableStream::from_native(ctx, IteratorSource(iter), None)?,
        )?
    } else if Buffer::is(&ctx, &value)? {
        Class::instance(
            ctx.clone(),
            ReadableStream::from_native(ctx, One::new(value), None)?,
        )?
    } else {
        todo!()
    };

    Ok(read)
}
