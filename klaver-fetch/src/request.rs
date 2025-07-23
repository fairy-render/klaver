use http::Extensions;
use klaver_base::{AbortSignal, Blob, streams::ReadableStream};
use rquickjs::{ArrayBuffer, Class, Ctx, JsLifetime, String, TypedArray, class::Trace};

use crate::{Headers, Method, body::BodyMixin};

#[rquickjs::class]
pub struct Request<'js> {
    #[qjs(get)]
    url: String<'js>,
    #[qjs(get)]
    method: Method,
    #[qjs(get)]
    headers: Class<'js, Headers<'js>>,
    body: BodyMixin<'js>,
    #[qjs(get)]
    signal: Class<'js, AbortSignal<'js>>,

    ext: Option<Extensions>,
}

impl<'js> Trace<'js> for Request<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.method.trace(tracer);
        self.headers.trace(tracer);
        self.body.trace(tracer);
    }
}

unsafe impl<'js> JsLifetime<'js> for Request<'js> {
    type Changed<'to> = Request<'to>;
}

#[rquickjs::methods]
impl<'js> Request<'js> {
    pub fn body_read(&self) -> bool {
        self.body.body_read()
    }

    pub fn body(&self, ctx: Ctx<'js>) -> rquickjs::Result<Option<Class<'js, ReadableStream<'js>>>> {
        self.body.body(&ctx)
    }

    pub async fn text(&self, ctx: Ctx<'js>) -> rquickjs::Result<std::string::String> {
        self.body.to_text(&ctx).await
    }

    pub async fn array_buffer(&self, ctx: Ctx<'js>) -> rquickjs::Result<ArrayBuffer<'js>> {
        todo!()
    }

    pub async fn bytes(&self, ctx: Ctx<'js>) -> rquickjs::Result<TypedArray<'js, u8>> {
        todo!()
    }

    pub async fn blob(&self, ctx: Ctx<'js>) -> rquickjs::Result<Blob<'js>> {
        todo!()
    }
}
