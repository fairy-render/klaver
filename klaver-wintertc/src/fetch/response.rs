use super::{
    Headers, StaticBody,
    body::{BodyMixin, JsBody},
    body_init::BodyInit,
    body_static::Body,
    form_data::FormData,
    response_init::ResponseInit,
};
use crate::{blob::Blob, streams::ReadableStream};
use http::{Extensions, StatusCode};
use klaver_core::{
    throw_if,
    value::{Pair, StringExt, TypedMultiMap, iterable::NativeIteratorExt},
};
use rquickjs::{
    ArrayBuffer, Class, Ctx, FromJs, JsLifetime, String, TypedArray, Value, class::Trace,
    prelude::Opt,
};

#[rquickjs::class]
pub struct Response<'js> {
    #[qjs(get)]
    pub headers: Class<'js, Headers<'js>>,
    pub status: StatusCode,
    pub body: BodyMixin<'js>,
    pub ext: Option<Extensions>,
    /// The response's URL, or the empty string for a `Response` built via `new Response()` -
    /// per <https://fetch.spec.whatwg.org/#dom-response-url>.
    pub url: String<'js>,
    /// Always `false`: this runtime doesn't itself follow redirects (whatever the underlying
    /// HTTP client backend does is opaque to it), so there's no way to honestly report `true`.
    pub redirected: bool,
    /// Either the caller-supplied `ResponseInit.statusText`, or the status's default reason
    /// phrase (e.g. `"OK"` for `200`) - stored rather than recomputed, since a custom
    /// `statusText` must be preserved verbatim, not just defaulted.
    pub status_text: String<'js>,
}

impl<'js> Trace<'js> for Response<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.headers.trace(tracer);
        self.body.trace(tracer);
        self.url.trace(tracer);
        self.status_text.trace(tracer);
    }
}

unsafe impl<'js> JsLifetime<'js> for Response<'js> {
    type Changed<'to> = Response<'to>;
}

impl<'js> Response<'js> {
    pub fn to_native(&mut self, ctx: &Ctx<'js>) -> rquickjs::Result<http::Response<JsBody<'js>>> {
        let mut builder = http::Response::builder().status(self.status.clone());

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

        Ok(req)
    }

    pub fn to_owned_native(
        &mut self,
        ctx: &Ctx<'js>,
    ) -> rquickjs::Result<http::Response<StaticBody>> {
        let mut builder = http::Response::builder().status(self.status.clone());

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

        Ok(req)
    }

    pub fn from_native(
        ctx: &Ctx<'js>,
        resp: http::Response<Body>,
        url: &str,
    ) -> rquickjs::Result<Response<'js>> {
        let (parts, body) = resp.into_parts();

        let body = BodyMixin::from(body);
        let headers = Headers::from_native(&ctx, parts.headers)?;

        let status: StatusCode = parts.status.into();
        let status_text = String::from_str(ctx.clone(), status.canonical_reason().unwrap_or(""))?;

        Ok(Response {
            headers,
            status,
            body,
            ext: parts.extensions.into(),
            url: String::from_str(ctx.clone(), url)?,
            redirected: false,
            status_text,
        })
    }
}

#[rquickjs::methods]
impl<'js> Response<'js> {
    #[qjs(constructor)]
    pub fn new(
        ctx: Ctx<'js>,
        Opt(body): Opt<Value<'js>>,
        Opt(init): Opt<ResponseInit<'js>>,
    ) -> rquickjs::Result<Response<'js>> {
        // `body`'s IDL type is `BodyInit? = BodyInit | null`: unlike an *omitted* argument
        // (which `Opt` already maps to `None`), an explicitly-passed `null` (or `undefined`)
        // must also mean "no body" here rather than failing to parse as a `BodyInit`.
        let body = match body {
            Some(value) if !value.is_null() && !value.is_undefined() => {
                Some(BodyInit::from_js(&ctx, value)?)
            }
            _ => None,
        };

        let (headers, status, status_text) = match init {
            Some(init) => init.build(ctx.clone())?,
            None => (
                Class::instance(ctx.clone(), Headers::new_native(ctx.clone())?)?,
                StatusCode::OK,
                String::from_str(ctx.clone(), StatusCode::OK.canonical_reason().unwrap_or(""))?,
            ),
        };

        let body = match body {
            Some(body) => body.to_body(&ctx, &headers)?,
            None => BodyMixin::empty(),
        };

        Ok(Response {
            headers,
            status,
            body,
            ext: None,
            url: String::from_str(ctx.clone(), "")?,
            redirected: false,
            status_text,
        })
    }

    #[qjs(get, rename = "bodyUsed")]
    pub fn body_used(&self) -> bool {
        self.body.body_used()
    }

    #[qjs(get)]
    pub fn url(&self) -> String<'js> {
        self.url.clone()
    }

    #[qjs(get)]
    pub fn redirected(&self) -> bool {
        self.redirected
    }

    pub fn body(&self, ctx: Ctx<'js>) -> rquickjs::Result<Option<Class<'js, ReadableStream<'js>>>> {
        self.body.body(&ctx)
    }

    #[qjs(get)]
    pub fn status(&self) -> u16 {
        self.status.as_u16()
    }

    #[qjs(get)]
    pub fn ok(&self) -> bool {
        self.status.is_success()
    }

    #[qjs(get, rename = "statusText")]
    pub fn status_text(&self) -> String<'js> {
        self.status_text.clone()
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

    pub async fn json(&self, ctx: Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        self.body.json(&ctx).await
    }

    #[qjs(rename = "formData")]
    pub async fn form_data(&self, ctx: Ctx<'js>) -> rquickjs::Result<FormData<'js>> {
        let content_type = self
            .headers
            .borrow()
            .get(ctx.clone(), String::from_str(ctx.clone(), "content-type")?)?;

        self.body.form_data(&ctx, content_type).await
    }

    /// Per <https://fetch.spec.whatwg.org/#dom-response-clone>: throws if the body has already
    /// been used or is locked, otherwise returns an independent `Response` with its own copy of
    /// the headers and (if a real body is present) a teed, independently-readable body.
    #[qjs(rename = "clone")]
    pub fn clone_response(&self, ctx: Ctx<'js>) -> rquickjs::Result<Response<'js>> {
        let headers_copy = TypedMultiMap::new(ctx.clone())?;
        for pair in self.headers.borrow().inner.entries()?.into_iter(&ctx) {
            let Pair(k, v) = pair?;
            headers_copy.append(&ctx, k, v)?;
        }

        Ok(Response {
            headers: Class::instance(
                ctx.clone(),
                Headers {
                    inner: headers_copy,
                },
            )?,
            status: self.status,
            body: self.body.try_clone(&ctx)?,
            ext: None,
            url: self.url.clone(),
            redirected: self.redirected,
            status_text: self.status_text.clone(),
        })
    }
}

klaver_core::create_export!(Response<'js>);

#[cfg(test)]
mod tests {
    use super::*;
    use klaver_core::value::FunctionExt;
    use rquickjs::{AsyncContext, AsyncRuntime, CatchResultExt, Function, class::JsClass};

    /// Runs `body` as the contents of an `async` function, with `Response`/`Headers` and their
    /// transitive dependencies registered as globals. `body` is expected to throw on failure
    /// (e.g. via a plain `if (...) throw ...`).
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
                    Response::NAME,
                    Class::<Response>::create_constructor(&ctx)?,
                )?;

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
    fn custom_status_text_is_preserved() {
        run(r#"
            const res = new Response("hi", { status: 200, statusText: "Super OK" });
            if (res.statusText !== "Super OK") throw new Error(`statusText was ${res.statusText}`);
        "#);
    }

    #[test]
    fn status_text_defaults_to_the_canonical_reason_phrase() {
        run(r#"
            const notFound = new Response(null, { status: 404 });
            if (notFound.statusText !== "Not Found") {
                throw new Error(`statusText was ${notFound.statusText}`);
            }

            const teapot = new Response(null, { status: 418 });
            if (teapot.statusText !== "I'm a teapot") {
                throw new Error(`statusText was ${teapot.statusText}`);
            }
        "#);
    }

    #[test]
    fn status_outside_200_to_599_throws() {
        run(r#"
            let threw = false;
            try {
                new Response(null, { status: 199 });
            } catch (err) {
                threw = true;
            }
            if (!threw) throw new Error("expected status 199 to throw");

            threw = false;
            try {
                new Response(null, { status: 600 });
            } catch (err) {
                threw = true;
            }
            if (!threw) throw new Error("expected status 600 to throw");
        "#);
    }

    #[test]
    fn defaults_to_status_200_and_empty_url() {
        run(r#"
            const res = new Response("hi");
            if (res.status !== 200) throw new Error(`status was ${res.status}`);
            if (!res.ok) throw new Error("expected ok to be true");
            if (res.statusText !== "OK") throw new Error(`statusText was ${res.statusText}`);
            if (res.url !== "") throw new Error(`url was ${res.url}`);
            if (res.redirected !== false) throw new Error("expected redirected to be false");
        "#);
    }

    #[test]
    fn ok_is_false_for_error_statuses() {
        run(r#"
            const res = new Response(null, { status: 404 });
            if (res.ok) throw new Error("expected ok to be false for a 404");
            if (res.status !== 404) throw new Error(`status was ${res.status}`);
        "#);
    }

    #[test]
    fn body_used_is_false_without_a_body_and_true_after_reading() {
        run(r#"
            const withoutBody = new Response();
            if (withoutBody.bodyUsed !== false) throw new Error("expected bodyUsed false without a body");
            const emptyText = await withoutBody.text();
            if (emptyText !== "") throw new Error(`expected empty text, got ${emptyText}`);

            const withBody = new Response("hi");
            if (withBody.bodyUsed !== false) throw new Error("expected bodyUsed false before reading");
            await withBody.text();
            if (withBody.bodyUsed !== true) throw new Error("expected bodyUsed true after reading");
        "#);
    }

    #[test]
    fn reading_the_body_twice_throws() {
        run(r#"
            const res = new Response("hi");
            await res.text();

            let threw = false;
            try {
                await res.text();
            } catch (err) {
                threw = true;
            }
            if (!threw) throw new Error("expected re-reading the body to throw");
        "#);
    }

    #[test]
    fn clone_produces_an_independently_readable_body_and_headers() {
        run(r#"
            const res = new Response("hello", { headers: { "X-Foo": "bar" } });

            const clone = res.clone();
            clone.headers.set("X-Foo", "changed");
            if (res.headers.get("x-foo") !== "bar") {
                throw new Error(`original headers mutated: ${res.headers.get("x-foo")}`);
            }

            const cloneText = await clone.text();
            if (cloneText !== "hello") throw new Error(`clone text was ${cloneText}`);

            const originalText = await res.text();
            if (originalText !== "hello") throw new Error(`original text was ${originalText}`);
        "#);
    }

    #[test]
    fn cloning_an_already_read_body_throws() {
        run(r#"
            const res = new Response("hi");
            await res.text();

            let threw = false;
            try {
                res.clone();
            } catch (err) {
                threw = true;
            }
            if (!threw) throw new Error("expected clone() of a used body to throw");
        "#);
    }

    /// `ext` (`http::Extensions`) is opaque, non-JS-facing metadata attached to a native
    /// request/response - not something a JS test can observe, so this is a plain Rust-level
    /// round-trip rather than the `run()`/JS harness the other tests use. `ext` is a one-time
    /// take (mirroring `Request`'s own `to_native`/`to_owned_native`), so each conversion needs
    /// its own freshly-`from_native`'d `Response`. Uses the async runtime (like `run()`) because
    /// even converting a body this small round-trips it through `ReadableStream`, which needs
    /// `ctx.spawn` support a bare sync `Context` doesn't provide.
    #[test]
    fn extensions_are_forwarded_through_to_native_and_to_owned_native() {
        #[derive(Clone)]
        struct Marker(u32);

        fn native_with_marker() -> http::Response<Body> {
            http::Response::builder()
                .status(200)
                .extension(Marker(42))
                .body(Body::empty())
                .unwrap()
        }

        futures::executor::block_on(async move {
            let rt = AsyncRuntime::new().unwrap();
            let ctx = AsyncContext::full(&rt).await.unwrap();

            ctx.async_with(async |ctx| {
                klaver_core::register(&ctx)?;

                let mut response = Response::from_native(&ctx, native_with_marker(), "")?;
                let native_out = response.to_native(&ctx)?;
                assert_eq!(native_out.extensions().get::<Marker>().map(|m| m.0), Some(42));

                let mut response = Response::from_native(&ctx, native_with_marker(), "")?;
                let native_out = response.to_owned_native(&ctx)?;
                assert_eq!(native_out.extensions().get::<Marker>().map(|m| m.0), Some(42));

                rquickjs::Result::Ok(())
            })
            .await
            .unwrap();
        });
    }
}
