use futures::io::Repeat;
use http::Extensions;
use klaver_base::{Blob, streams::ReadableStream};
use reggie::Body;
use rquickjs::{ArrayBuffer, Class, Ctx, JsLifetime, String, TypedArray, class::Trace};

use crate::{
    Headers,
    body::{BodyMixin, JsBody},
};

#[rquickjs::class]
pub struct Response<'js> {
    #[qjs(get)]
    pub headers: Class<'js, Headers<'js>>,
    #[qjs(get)]
    pub status: u16,
    pub body: BodyMixin<'js>,
    pub ext: Option<Extensions>,
}

impl<'js> Trace<'js> for Response<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.headers.trace(tracer);
        self.body.trace(tracer);
    }
}

unsafe impl<'js> JsLifetime<'js> for Response<'js> {
    type Changed<'to> = Response<'to>;
}

impl<'js> Response<'js> {
    pub fn to_native(&self, ctx: &Ctx<'js>) -> rquickjs::Result<http::Response<JsBody<'js>>> {
        todo!()
    }

    pub fn from_native(
        ctx: &Ctx<'js>,
        resp: http::Response<Body>,
    ) -> rquickjs::Result<Response<'js>> {
        let (parts, body) = resp.into_parts();

        let body = BodyMixin::from(body);
        let headers = Headers::from_native(&ctx, parts.headers)?;

        Ok(Response {
            headers,
            status: parts.status.into(),
            body,
            ext: parts.extensions.into(),
        })
    }
}

impl<'js> Response<'js> {
    pub fn body(&self, ctx: Ctx<'js>) -> rquickjs::Result<Option<Class<'js, ReadableStream<'js>>>> {
        self.body.body(&ctx)
    }

    pub async fn text(&self, ctx: Ctx<'js>) -> rquickjs::Result<String<'js>> {
        self.body.to_text(&ctx).await
    }

    pub async fn array_buffer(&self, ctx: Ctx<'js>) -> rquickjs::Result<ArrayBuffer<'js>> {
        self.body.array_buffer(&ctx).await
    }

    pub async fn bytes(&self, ctx: Ctx<'js>) -> rquickjs::Result<TypedArray<'js, u8>> {
        self.body.bytes(&ctx).await
    }

    pub async fn blob(&self, ctx: Ctx<'js>) -> rquickjs::Result<Blob<'js>> {
        let content_type = self
            .headers
            .borrow()
            .get(ctx.clone(), String::from_str(ctx.clone(), "content-type")?)?;

        self.body.blob(&ctx, content_type).await
    }
}
