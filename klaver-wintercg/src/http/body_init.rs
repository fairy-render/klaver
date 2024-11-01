use rquickjs::{class::Trace, Class, Ctx, FromJs};
use rquickjs_util::Buffer;

use crate::streams::{One, ReadableStream};

#[derive(Trace)]
pub enum BodyInit<'js> {
    Buffer(Buffer<'js>),
    String(rquickjs::String<'js>),
    Stream(Class<'js, ReadableStream<'js>>),
}

impl<'js> BodyInit<'js> {
    pub fn to_stream(self, ctx: &Ctx<'js>) -> rquickjs::Result<Class<'js, ReadableStream<'js>>> {
        match self {
            BodyInit::Buffer(buffer) => ReadableStream::from_native(ctx.clone(), One::new(buffer)),
            BodyInit::String(str) => ReadableStream::from_native(ctx.clone(), One::new(str)),
            BodyInit::Stream(stream) => Ok(stream.clone()),
        }
    }
}

impl<'js> FromJs<'js> for BodyInit<'js> {
    fn from_js(
        ctx: &rquickjs::prelude::Ctx<'js>,
        value: rquickjs::Value<'js>,
    ) -> rquickjs::Result<Self> {
        let body = if Buffer::is(ctx, &value)? {
            BodyInit::Buffer(Buffer::from_js(ctx, value)?)
        } else if value.is_string() {
            let str = value
                .try_into_string()
                .map_err(|_| rquickjs::Error::new_from_js("value", "string"))?;
            BodyInit::String(str)
        } else if ReadableStream::is(&value) {
            BodyInit::Stream(Class::<ReadableStream>::from_js(ctx, value)?)
        } else {
            return Err(rquickjs::Error::new_from_js("value", "string or buffer"));
        };

        Ok(body)
    }
}
