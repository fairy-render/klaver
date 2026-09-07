use crate::{abort_controller::AbortSignal, blob::Blob, streams::ReadableStream};
use http::Extensions;
use klaver_core::{
    throw, throw_if,
    value::{Pair, StringExt, TypedMultiMap, iterable::NativeIteratorExt},
};
use rquickjs::{
    ArrayBuffer, Class, Coerced, Ctx, FromJs, JsLifetime, String, TypedArray, Value, class::Trace,
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

/// The `input` argument to the `Request` constructor: `RequestInfo = Request | string` per
/// <https://fetch.spec.whatwg.org/#requestinfo>. Anything else (e.g. a `URL` object) is
/// coerced to a string, same as before this distinguished `Request` instances specially.
pub enum RequestInfo<'js> {
    Request(Class<'js, Request<'js>>),
    String(String<'js>),
}

impl<'js> FromJs<'js> for RequestInfo<'js> {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        if let Ok(req) = Class::<'js, Request<'js>>::from_js(ctx, value.clone()) {
            Ok(RequestInfo::Request(req))
        } else {
            let Coerced(s) = Coerced::<String<'js>>::from_js(ctx, value)?;
            Ok(RequestInfo::String(s))
        }
    }
}

impl<'js> Request<'js> {
    pub fn to_native(
        &mut self,
        ctx: &Ctx<'js>,
    ) -> rquickjs::Result<(
        http::Request<JsBody<'js>>,
        Option<Class<'js, AbortSignal<'js>>>,
    )> {
        let mut builder = http::Request::builder()
            .method(self.method.0.clone())
            .uri(self.url.str_ref()?.as_str());

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
        let mut builder = http::Request::builder()
            .method(self.method.0.clone())
            .uri(self.url.str_ref()?.as_str());

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
        input: RequestInfo<'js>,
        init: Opt<RequestInit<'js>>,
    ) -> rquickjs::Result<Request<'js>> {
        let init = init.0;

        // Whether `init` will supply its own body - if so, there's no need to (and, per spec,
        // no need to require-unused-and-clone) `input`'s body below.
        let init_overrides_body = init.as_ref().is_some_and(|i| i.body.is_some());

        // If `input` is an existing `Request`, it seeds the url/method/headers/signal/body,
        // each of which `init` may then individually override - per
        // <https://fetch.spec.whatwg.org/#dom-request>. A plain string `input` just becomes the
        // url, with everything else left for `init` (or its defaults) to supply.
        let (url, base_method, base_headers, base_signal, base_body) = match input {
            RequestInfo::String(url) => {
                // Per <https://fetch.spec.whatwg.org/#dom-request>: "Let parsedURL be the
                // result of parsing input... If parsedURL is failure, then throw a TypeError."
                // The request's url is the *parsed and re-serialized* URL, not the raw input
                // string (so e.g. a missing default port or inconsistent casing is normalized
                // away, matching what `new URL(input).href` would produce).
                let parsed = throw_if!(ctx, url::Url::parse(&url.to_string()?));
                let url = String::from_str(ctx.clone(), parsed.as_str())?;
                (url, None, None, None, None)
            }
            RequestInfo::Request(req) => {
                let req = req.borrow();

                // Per spec, reusing an already-disturbed/locked body (without `init` supplying
                // a replacement) is an error, rather than silently producing a body-less
                // request. Unlike the spec's exact "the two requests share one disturbed
                // stream" behavior, this tees the body so both `input` and the new `Request`
                // stay independently readable - the same simplification `clone()` makes.
                let base_body = if init_overrides_body {
                    None
                } else {
                    Some(req.body.try_clone(&ctx)?)
                };

                (
                    req.url.clone(),
                    Some(req.method.clone()),
                    Some(req.headers.clone()),
                    req.signal.clone(),
                    base_body,
                )
            }
        };

        let (method, headers, signal, body) = match init {
            Some(opts) => (opts.method, opts.headers, opts.signal, opts.body),
            None => (None, None, None, None),
        };

        let method = method.or(base_method).unwrap_or(Method(http::Method::GET));

        let headers = match headers {
            Some(ret) => ret.inner,
            None => match base_headers {
                // Copy rather than alias `input`'s headers - mutating the new `Request`'s
                // headers must not also mutate `input`'s.
                Some(existing) => {
                    let copy = TypedMultiMap::new(ctx.clone())?;
                    for pair in existing.borrow().inner.entries()?.into_iter(&ctx) {
                        let Pair(k, v) = pair?;
                        copy.append(&ctx, k, v)?;
                    }
                    Class::instance(ctx.clone(), Headers { inner: copy })?
                }
                None => Class::instance(ctx.clone(), Headers::new_native(ctx.clone())?)?,
            },
        };

        let signal = signal.or(base_signal);

        // Per <https://fetch.spec.whatwg.org/#dom-request>: "If init["body"] exists and
        // request's method is `GET` or `HEAD`, then throw a TypeError."
        if body.is_some() && matches!(method.as_str(), "GET" | "HEAD") {
            throw!(@type ctx, "Request with GET/HEAD method cannot have a body")
        }

        let body = match body {
            Some(body) => body.to_body(&ctx, &headers)?,
            None => base_body.unwrap_or_else(BodyMixin::empty),
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

    #[qjs(get, rename = "bodyUsed")]
    pub fn body_used(&self) -> bool {
        self.body.body_used()
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

    /// Per <https://fetch.spec.whatwg.org/#dom-request-clone>: throws if the body has already
    /// been used or is locked, otherwise returns an independent `Request` with its own copy of
    /// the headers and (if a real body is present) a teed, independently-readable body.
    #[qjs(rename = "clone")]
    pub fn clone_request(&self, ctx: Ctx<'js>) -> rquickjs::Result<Request<'js>> {
        let headers_copy = TypedMultiMap::new(ctx.clone())?;
        for pair in self.headers.borrow().inner.entries()?.into_iter(&ctx) {
            let Pair(k, v) = pair?;
            headers_copy.append(&ctx, k, v)?;
        }

        Ok(Request {
            url: self.url.clone(),
            method: self.method.clone(),
            headers: Class::instance(
                ctx.clone(),
                Headers {
                    inner: headers_copy,
                },
            )?,
            body: self.body.try_clone(&ctx)?,
            signal: self.signal.clone(),
            ext: None,
        })
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
    fn invalid_url_throws() {
        run(r#"
            let threw = false;
            try {
                new Request("not a url");
            } catch (err) {
                threw = true;
            }
            if (!threw) throw new Error("expected an invalid url to throw");
        "#);
    }

    #[test]
    fn url_is_parsed_and_normalized() {
        run(r#"
            // Default port for https (443) is dropped, matching `new URL(...).href`.
            const req = new Request("HTTPS://example.com:443/a");
            if (req.url !== "https://example.com/a") throw new Error(`url was ${req.url}`);
        "#);
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

    #[test]
    fn get_request_with_a_body_throws() {
        run(r#"
            let threw = false;
            try {
                new Request("https://example.com", { method: "GET", body: "hi" });
            } catch (err) {
                threw = true;
            }
            if (!threw) throw new Error("expected GET with a body to throw");
        "#);
    }

    #[test]
    fn head_request_with_a_body_throws() {
        run(r#"
            let threw = false;
            try {
                new Request("https://example.com", { method: "HEAD", body: "hi" });
            } catch (err) {
                threw = true;
            }
            if (!threw) throw new Error("expected HEAD with a body to throw");
        "#);
    }

    #[test]
    fn method_defaults_to_get_and_is_normalized() {
        run(r#"
            const req1 = new Request("https://example.com");
            if (req1.method !== "GET") throw new Error(`default method was ${req1.method}`);

            const req2 = new Request("https://example.com", { method: "post" });
            if (req2.method !== "POST") throw new Error(`normalized method was ${req2.method}`);
        "#);
    }

    #[test]
    fn body_used_is_false_without_a_body_and_true_after_reading() {
        run(r#"
            const withoutBody = new Request("https://example.com");
            if (withoutBody.bodyUsed !== false) throw new Error("expected bodyUsed false without a body");
            const emptyText = await withoutBody.text();
            if (emptyText !== "") throw new Error(`expected empty text, got ${emptyText}`);

            const withBody = new Request("https://example.com", { method: "POST", body: "hi" });
            if (withBody.bodyUsed !== false) throw new Error("expected bodyUsed false before reading");
            await withBody.text();
            if (withBody.bodyUsed !== true) throw new Error("expected bodyUsed true after reading");
        "#);
    }

    #[test]
    fn explicit_null_body_is_treated_as_no_body() {
        run(r#"
            const req = new Request("https://example.com", { method: "GET", body: null });
            if (req.bodyUsed !== false) throw new Error("expected bodyUsed to be false");
            const text = await req.text();
            if (text !== "") throw new Error(`expected empty text, got ${text}`);
        "#);
    }

    #[test]
    fn reading_the_body_twice_throws() {
        run(r#"
            const req = new Request("https://example.com", { method: "POST", body: "hi" });
            await req.text();

            let threw = false;
            try {
                await req.text();
            } catch (err) {
                threw = true;
            }
            if (!threw) throw new Error("expected re-reading the body to throw");
        "#);
    }

    #[test]
    fn clone_produces_an_independently_readable_body_and_headers() {
        run(r#"
            const req = new Request("https://example.com", {
                method: "POST",
                body: "hello",
                headers: { "X-Foo": "bar" },
            });

            const clone = req.clone();
            clone.headers.set("X-Foo", "changed");
            if (req.headers.get("x-foo") !== "bar") {
                throw new Error(`original headers mutated: ${req.headers.get("x-foo")}`);
            }

            const cloneText = await clone.text();
            if (cloneText !== "hello") throw new Error(`clone text was ${cloneText}`);

            const originalText = await req.text();
            if (originalText !== "hello") throw new Error(`original text was ${originalText}`);
        "#);
    }

    #[test]
    fn cloning_an_already_read_body_throws() {
        run(r#"
            const req = new Request("https://example.com", { method: "POST", body: "hi" });
            await req.text();

            let threw = false;
            try {
                req.clone();
            } catch (err) {
                threw = true;
            }
            if (!threw) throw new Error("expected clone() of a used body to throw");
        "#);
    }

    #[test]
    fn constructing_from_an_existing_request_copies_its_fields() {
        run(r#"
            const original = new Request("https://example.com/a", {
                method: "POST",
                body: "hello",
                headers: { "X-Foo": "bar" },
            });

            const copy = new Request(original);
            if (copy.url !== "https://example.com/a") throw new Error(`url was ${copy.url}`);
            if (copy.method !== "POST") throw new Error(`method was ${copy.method}`);
            if (copy.headers.get("x-foo") !== "bar") throw new Error(`header was ${copy.headers.get("x-foo")}`);

            const copyText = await copy.text();
            if (copyText !== "hello") throw new Error(`copy text was ${copyText}`);

            // The original must still be independently readable - not "used up" by the copy.
            const originalText = await original.text();
            if (originalText !== "hello") throw new Error(`original text was ${originalText}`);
        "#);
    }

    #[test]
    fn constructing_from_an_existing_request_lets_init_override_fields() {
        run(r#"
            const original = new Request("https://example.com/a", {
                method: "POST",
                body: "hello",
                headers: { "X-Foo": "bar" },
            });

            const overridden = new Request(original, {
                method: "PUT",
                body: "goodbye",
                headers: { "X-Foo": "baz" },
            });

            if (overridden.url !== "https://example.com/a") throw new Error(`url was ${overridden.url}`);
            if (overridden.method !== "PUT") throw new Error(`method was ${overridden.method}`);
            if (overridden.headers.get("x-foo") !== "baz") {
                throw new Error(`header was ${overridden.headers.get("x-foo")}`);
            }

            const text = await overridden.text();
            if (text !== "goodbye") throw new Error(`text was ${text}`);

            // Overriding the body means the original is untouched and still independently readable.
            const originalText = await original.text();
            if (originalText !== "hello") throw new Error(`original text was ${originalText}`);
        "#);
    }

    #[test]
    fn constructing_from_an_existing_request_headers_copy_is_independent() {
        run(r#"
            const original = new Request("https://example.com/a", { headers: { "X-Foo": "bar" } });
            const copy = new Request(original);
            copy.headers.set("X-Foo", "changed");
            if (original.headers.get("x-foo") !== "bar") {
                throw new Error(`original headers mutated: ${original.headers.get("x-foo")}`);
            }
        "#);
    }
}
