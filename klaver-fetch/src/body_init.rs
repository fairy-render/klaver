use klaver_base::streams::{ReadableStream, readable::One};
use rquickjs::{Class, Ctx, FromJs, class::Trace};
use rquickjs_util::Buffer;

use crate::{Headers, body::BodyMixin};

#[derive(Trace)]
pub enum BodyInit<'js> {
    Buffer(Buffer<'js>),
    String(rquickjs::String<'js>),
    Stream(Class<'js, ReadableStream<'js>>),
}

impl<'js> BodyInit<'js> {
    pub fn to_body(
        self,
        ctx: &Ctx<'js>,
        headers: &Class<'js, Headers<'js>>,
    ) -> rquickjs::Result<BodyMixin<'js>> {
        match self {
            BodyInit::Buffer(buffer) => Ok(buffer.array_buffer()?.into()),
            BodyInit::String(str) => todo!(),
            BodyInit::Stream(stream) => Ok(stream.into()),
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
