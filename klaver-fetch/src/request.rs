use http::Extensions;
use klaver_base::{AbortSignal, Blob, create_export, streams::ReadableStream};
use klaver_util::{NativeIteratorExt, StringExt, throw_if};
use rquickjs::{
    ArrayBuffer, Class, Coerced, Ctx, JsLifetime, String, TypedArray, Value, class::Trace,
    prelude::Opt,
};

use crate::{
    Headers, Method, StaticBody,
    body::{BodyMixin, JsBody},
    body_static::Body,
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
        &mut self,
        ctx: &Ctx<'js>,
    ) -> rquickjs::Result<(
        http::Request<JsBody<'js>>,
        Option<Class<'js, AbortSignal<'js>>>,
    )> {
        let mut builder = http::Request::builder().uri(self.url.str_ref()?.as_str());

        let headers = self.headers.borrow();

        for pair in headers.inner.entries()?.into_iter(ctx) {
            let pair = pair?;
            builder = builder.header(pair.0.str_ref()?.as_str(), pair.1.str_ref()?.as_str());
        }

        let body = self.body.to_native_body(&ctx)?;

        let mut req = throw_if!(ctx, builder.body(body));
        if let Some(ext) = self.ext.take() {
            *req.extensions_mut() = ext;
        }

        Ok((req, self.signal.clone()))
    }

    pub fn to_owned_native(
        &mut self,
        ctx: &Ctx<'js>,
    ) -> rquickjs::Result<(
        http::Request<StaticBody>,
        Option<Class<'js, AbortSignal<'js>>>,
    )> {
        let mut builder = http::Request::builder().uri(self.url.str_ref()?.as_str());

        let headers = self.headers.borrow();

        for pair in headers.inner.entries()?.into_iter(ctx) {
            let pair = pair?;
            builder = builder.header(pair.0.str_ref()?.as_str(), pair.1.str_ref()?.as_str());
        }

        let body = self.body.to_native_static_body(&ctx)?;

        let mut req = throw_if!(ctx, builder.body(body));
        if let Some(ext) = self.ext.take() {
            *req.extensions_mut() = ext;
        }

        Ok((req, self.signal.clone()))
    }

    pub fn from_native(
        ctx: &Ctx<'js>,
        resp: http::Request<Body>,
    ) -> rquickjs::Result<Request<'js>> {
        let (parts, body) = resp.into_parts();

        let body = BodyMixin::from(body);
        let headers = Headers::from_native(&ctx, parts.headers)?;

        let url = String::from_str(ctx.clone(), &parts.uri.to_string())?;

        Ok(Request {
            headers,
            body,
            method: Method(parts.method),
            url,
            signal: None,
            ext: parts.extensions.into(),
        })
    }
}

#[rquickjs::methods]
impl<'js> Request<'js> {
    #[qjs(constructor)]
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

    pub async fn json(&self, ctx: Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        self.body.json(&ctx).await
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
