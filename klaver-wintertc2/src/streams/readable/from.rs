use klaver_core::value::{
    Buffer, async_iterator::AsyncIterable, async_iterator::is_async_iterable, iterable::JsIterable,
    iterable::is_iteratable,
};
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
        let iter = value.get::<JsIterable>()?.iterator()?;
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
