use klaver_base::{Blob, streams::ReadableStream};
use klaver_util::{Buffer, StringExt};
use rquickjs::{ArrayBuffer, Class, Coerced, Ctx, FromJs, String, class::Trace};

use crate::{Headers, URLSearchParams, body::BodyMixin};

#[derive(Trace)]
pub enum BodyInit<'js> {
    Buffer(Buffer<'js>),
    String(rquickjs::String<'js>),
    UrlSearchParam(Class<'js, URLSearchParams<'js>>),
    Blob(Class<'js, Blob<'js>>),
    Stream(Class<'js, ReadableStream<'js>>),
}

impl<'js> BodyInit<'js> {
    pub fn to_body(
        self,
        ctx: &Ctx<'js>,
        headers: &Class<'js, Headers<'js>>,
    ) -> rquickjs::Result<BodyMixin<'js>> {
        let content_type = String::from_str(ctx.clone(), "content-type")?;

        match self {
            BodyInit::Buffer(buffer) => Ok(buffer.array_buffer()?.into()),
            BodyInit::String(str) => {
                let buffer = ArrayBuffer::new_copy(ctx.clone(), str.str_ref()?.as_bytes())?;
                Ok(buffer.into())
            }
            BodyInit::UrlSearchParam(params) => {
                let body = params.borrow().to_string(ctx.clone())?;
                let buffer = ArrayBuffer::new(ctx.clone(), body)?;
                if !headers.borrow().has(ctx.clone(), content_type.clone())? {
                    headers.borrow_mut().append(
                        ctx.clone(),
                        content_type,
                        Coerced(String::from_str(
                            ctx.clone(),
                            "application/x-www-form-urlencoded",
                        )?),
                    )?;
                }
                Ok(buffer.into())
            }
            BodyInit::Blob(blob) => {
                let buffer = blob.borrow().buffer.clone();
                if let Some(ty) = blob.borrow().ty.clone() {
                    if !headers.borrow().has(ctx.clone(), content_type.clone())? {
                        headers
                            .borrow_mut()
                            .append(ctx.clone(), content_type, Coerced(ty))?;
                    }
                }

                Ok(buffer.into())
            }
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
        } else if let Ok(params) = value.get::<Class<'js, URLSearchParams<'js>>>() {
            BodyInit::UrlSearchParam(params)
        } else if let Ok(blob) = value.get::<Class<'js, Blob<'js>>>() {
            BodyInit::Blob(blob)
        } else {
            return Err(rquickjs::Error::new_from_js("value", "string or buffer"));
        };

        Ok(body)
    }
}
