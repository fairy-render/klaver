use http::Extensions;
use klaver_base::{AbortSignal, Blob, create_export, streams::ReadableStream};
use reggie::Body;
use rquickjs::{
    ArrayBuffer, Class, Coerced, Ctx, JsLifetime, String, TypedArray, class::Trace, prelude::Opt,
};
use rquickjs_util::{StringRef, throw_if, util::StringExt};

use crate::{
    Headers, Method,
    body::{BodyMixin, JsBody},
    request_init::RequestInit,
};

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
    signal: Option<Class<'js, AbortSignal<'js>>>,

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

impl<'js> Request<'js> {
    pub fn to_native(
        &self,
        ctx: &Ctx<'js>,
    ) -> rquickjs::Result<(
        http::Request<JsBody<'js>>,
        Option<Class<'js, AbortSignal<'js>>>,
    )> {
        let mut builder = http::Request::builder().uri(self.url.str_ref()?.as_str());

        let headers = self.headers.borrow();

        for pair in headers.inner.entries()? {
            let pair = pair?;
            builder = builder.header(pair.key.str_ref()?.as_str(), pair.value.str_ref()?.as_str());
        }

        let body = self.body.to_native_body(&ctx)?;

        let req = throw_if!(ctx, builder.body(body));

        Ok((req, self.signal.clone()))
    }
}

#[rquickjs::methods]
impl<'js> Request<'js> {
    pub fn new(
        ctx: Ctx<'js>,
        Coerced(url): Coerced<String<'js>>,
        init: Opt<RequestInit<'js>>,
    ) -> rquickjs::Result<Request<'js>> {
        let (method, headers, signal, body) = if let Some(opts) = init.0 {
            (opts.method, opts.headers, opts.signal, opts.body)
        } else {
            (None, None, None, None)
        };

        let headers = match headers {
            Some(ret) => ret.inner,
            None => Class::instance(ctx.clone(), Headers::new_native(ctx.clone())?)?,
        };

        let method = method.unwrap_or(Method(http::Method::GET));

        let body = if let Some(body) = body {
            let body: BodyMixin<'js> = body.to_body(&ctx, &headers)?;
            body
        } else {
            BodyMixin::empty()
        };

        Ok(Request {
            url,
            method,
            headers,
            body,
            signal,
            ext: None,
        })
    }

    #[qjs(get, rename = "bodyRead")]
    pub fn body_read(&self) -> bool {
        self.body.body_read()
    }

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

create_export!(Request<'js>);
