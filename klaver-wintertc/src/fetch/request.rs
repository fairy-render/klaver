use crate::{abort_controller::AbortSignal, blob::Blob, streams::ReadableStream};
use http::Extensions;
use klaver_core::{throw_if, value::StringExt, value::iterable::NativeIteratorExt};
use rquickjs::{
    ArrayBuffer, Class, Coerced, Ctx, JsLifetime, String, TypedArray, Value, class::Trace,
    prelude::Opt,
};

use super::{
    Headers, Method, StaticBody,
    body::{BodyMixin, JsBody},
    body_static::Body,
    form_data::FormData,
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
        self.url.trace(tracer);
        self.method.trace(tracer);
        self.headers.trace(tracer);
        self.body.trace(tracer);
        self.signal.trace(tracer);
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

    #[qjs(rename = "formData")]
    pub async fn form_data(&self, ctx: Ctx<'js>) -> rquickjs::Result<FormData<'js>> {
        let content_type = self
            .headers
            .borrow()
            .get(ctx.clone(), String::from_str(ctx.clone(), "content-type")?)?;

        self.body.form_data(&ctx, content_type).await
    }
}

klaver_core::create_export!(Request<'js>);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blob::{File, NativeBlob};
    use crate::fetch::URLSearchParams;
    use klaver_core::value::{FunctionExt, iterable::IterableProtocol};
    use klaver_core::Subclass;
    use rquickjs::{AsyncContext, AsyncRuntime, CatchResultExt, Function, class::JsClass};

    /// Runs `body` as the contents of an `async` function, with `Request`/`FormData`/`Blob`/
    /// `File` and their transitive dependencies registered as globals. `body` is expected to
    /// throw on failure (e.g. via a plain `if (...) throw ...`).
    fn run(body: &str) {
        futures::executor::block_on(async move {
            let rt = AsyncRuntime::new().unwrap();
            let ctx = AsyncContext::full(&rt).await.unwrap();

            ctx.async_with(async |ctx| {
                // `Headers` (via `TypedMultiMap`) goes through `BasePrimordials`, which needs
                // the `$runtime` Core global that `klaver_core::register` sets up - normally
                // done by the `Environ`/`Vm` builder, but this test drives a bare `Context`.
                klaver_core::register(&ctx)?;

                ctx.globals()
                    .set(Headers::NAME, Class::<Headers>::create_constructor(&ctx)?)?;
                ctx.globals().set(
                    URLSearchParams::NAME,
                    Class::<URLSearchParams>::create_constructor(&ctx)?,
                )?;
                ctx.globals()
                    .set(Blob::NAME, Class::<Blob>::create_constructor(&ctx)?)?;
                Blob::add_blob_prototype(&ctx)?;
                ctx.globals()
                    .set(File::NAME, Class::<File>::create_constructor(&ctx)?)?;
                File::inherit(&ctx)?;
                ctx.globals()
                    .set(FormData::NAME, Class::<FormData>::create_constructor(&ctx)?)?;
                FormData::add_iterable_prototype(&ctx)?;
                ctx.globals()
                    .set(Request::NAME, Class::<Request>::create_constructor(&ctx)?)?;

                let test_fn: Function = ctx.eval(format!("(async () => {{\n{body}\n}})"))?;

                if let Err(err) = test_fn.call_async::<_, ()>(()).await.catch(&ctx) {
                    panic!("{err}");
                }

                rquickjs::Result::Ok(())
            })
            .await
            .unwrap();
        });
    }

    #[test]
    fn form_data_round_trips_through_request_body() {
        run(r#"
            const fd = new FormData();
            fd.append("name", "klaver");
            fd.append("file", new Blob(["hello"], { type: "text/plain" }), "hello.txt");

            const req = new Request("https://example.com", { method: "POST", body: fd });

            const contentType = req.headers.get("content-type");
            if (!contentType || !contentType.startsWith("multipart/form-data")) {
                throw new Error(`content-type was ${contentType}`);
            }

            const parsed = await req.formData();
            if (parsed.get("name") !== "klaver") {
                throw new Error(`name was ${parsed.get("name")}`);
            }

            const file = parsed.get("file");
            if (!(file instanceof File)) throw new Error("expected a File instance");
            if (!(file instanceof Blob)) throw new Error("expected File to be a Blob");
            if (file.name !== "hello.txt") throw new Error(`file name was ${file.name}`);

            const text = await file.text();
            if (text !== "hello") throw new Error(`file text was ${text}`);
        "#);
    }

    #[test]
    fn url_encoded_body_parses_as_form_data() {
        run(r#"
            const req = new Request("https://example.com", {
                method: "POST",
                body: new URLSearchParams({ a: "1", b: "2" }),
            });

            const parsed = await req.formData();
            if (parsed.get("a") !== "1") throw new Error(`a was ${parsed.get("a")}`);
            if (parsed.get("b") !== "2") throw new Error(`b was ${parsed.get("b")}`);
        "#);
    }
}
