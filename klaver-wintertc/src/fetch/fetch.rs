use crate::{abort_controller::AbortSignal, events::Emitter};
use futures::{FutureExt, StreamExt};
use klaver_core::{StringExt, throw};
use rquickjs::{Class, Coerced, Ctx, FromJs, String, prelude::Opt};

use super::{
    Url, body::JsBody, client::Client,
    request::{Request, RequestInfo},
    request_init::RequestInit,
    response::Response,
};

pub enum FetchInit<'js> {
    Request(Class<'js, Request<'js>>),
    Url(Class<'js, Url<'js>>),
    String(String<'js>),
}

impl<'js> FetchInit<'js> {
    pub fn to_native_request(
        self,
        ctx: &Ctx<'js>,
        _client: &Client,
        init: Option<RequestInit<'js>>,
    ) -> rquickjs::Result<(
        http::Request<JsBody<'js>>,
        Option<Class<'js, AbortSignal<'js>>>,
    )> {
        match self {
            // Per <https://fetch.spec.whatwg.org/#dom-global-fetch>, `input`'s `init` overrides
            // still apply even when `input` is itself a `Request` object - so unless `init` is
            // empty, this must go through the same "construct a `Request` from a `Request`"
            // override logic `new Request(input, init)` uses, not just reuse `input` verbatim.
            Self::Request(req) if init.is_some() => {
                let mut req = Request::new(ctx.clone(), RequestInfo::Request(req), Opt(init))?;
                req.to_native(ctx)
            }
            Self::Request(req) => req.borrow_mut().to_native(ctx),
            Self::String(url) => {
                let mut req = Request::new(ctx.clone(), RequestInfo::String(url), Opt(init))?;
                req.to_native(ctx)
            }
            Self::Url(url) => {
                let Coerced(url) = Coerced::from_js(ctx, url.into_value())?;
                let mut req = Request::new(ctx.clone(), RequestInfo::String(url), Opt(init))?;
                req.to_native(ctx)
            }
        }
    }
}

impl<'js> FromJs<'js> for FetchInit<'js> {
    fn from_js(ctx: &Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        if let Ok(req) = Class::<'js, Request<'js>>::from_value(&value) {
            Ok(FetchInit::Request(req))
        } else if let Ok(string) = String::from_value(value.clone()) {
            Ok(FetchInit::String(string))
        } else if let Ok(url) = Class::<'js, Url<'js>>::from_value(&value) {
            Ok(FetchInit::Url(url))
        } else {
            throw!(@type ctx, "Expected a request object, string or a url")
        }
    }
}

pub async fn fetch<'js>(
    ctx: Ctx<'js>,
    url: FetchInit<'js>,
    init: Opt<RequestInit<'js>>,
) -> rquickjs::Result<Response<'js>> {
    let client = Client::from_ctx(&ctx)?;

    let (req, signal) = url.to_native_request(&ctx, &client, init.0)?;
    let request_url = req.uri().to_string();

    if let Some(signal) = &signal {
        // Per <https://fetch.spec.whatwg.org/#dom-global-fetch>: if the signal is already
        // aborted before we even start, reject immediately (without touching the network) with
        // the same reason `AbortSignal.reason` exposes.
        if signal.borrow().aborted {
            let reason = signal
                .borrow()
                .reason
                .clone()
                .expect("aborted signal always has a reason");
            return Err(ctx.throw(reason));
        }
    }

    let future = client.send(&ctx, req);

    if let Some(signal) = signal {
        let (sx, mut rx) = futures::channel::mpsc::channel(1);

        signal.borrow_mut().add_native_listener(
            String::from_str(ctx.clone(), "abort")?.str_ref()?.into(),
            sx,
        );

        futures::select! {
            ret = future.fuse() => {

                match ret {
                    Ok(resp) => Response::from_native(&ctx, resp, &request_url),
                    Err(err) => Err(err)
                }
            }
            _ = rx.next().fuse() => {
                // Per <https://fetch.spec.whatwg.org/#dom-global-fetch>, an aborted fetch
                // rejects with the signal's abort reason (an `AbortError` `DOMException` by
                // default), not a generic error.
                let reason = signal
                    .borrow()
                    .reason
                    .clone()
                    .expect("abort() always sets a reason before dispatching the event");
                Err(ctx.throw(reason))
            }
        }
    } else {
        let resp = future.await?;

        Response::from_native(&ctx, resp, &request_url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        abort_controller::AbortController,
        dom_exception::DOMException,
        events::{Event, EventTarget, NativeEvent},
    };
    use futures::future::LocalBoxFuture;
    use klaver_core::{Subclass, value::FunctionExt};
    use rquickjs::{
        AsyncContext, AsyncRuntime, CatchResultExt, Function,
        class::JsClass,
        prelude::{Async, Func},
    };

    use super::super::{Body, Headers, body_static::to_bytes, client::LocalClient, set_local_client};

    /// Resolves every request with a canned 201 response, echoing nothing about the request
    /// itself - just enough to exercise `fetch()`'s success path.
    struct EchoClient;

    impl LocalClient for EchoClient {
        fn send<'js, 'a>(
            &'a self,
            _ctx: &'a Ctx<'js>,
            _req: http::Request<JsBody<'js>>,
        ) -> LocalBoxFuture<'a, rquickjs::Result<http::Response<Body>>> {
            Box::pin(async move {
                Ok(http::Response::builder()
                    .status(201)
                    .header("content-type", "text/plain")
                    .body(Body::from("hello"))
                    .expect("valid response"))
            })
        }
    }

    /// Echoes the request's method and body text back as response headers, so a test can
    /// observe what `fetch()` actually sent - used to verify `init` overrides are applied even
    /// when `input` is itself a `Request` object.
    struct EchoMethodClient;

    impl LocalClient for EchoMethodClient {
        fn send<'js, 'a>(
            &'a self,
            _ctx: &'a Ctx<'js>,
            req: http::Request<JsBody<'js>>,
        ) -> LocalBoxFuture<'a, rquickjs::Result<http::Response<Body>>> {
            Box::pin(async move {
                let method = req.method().to_string();
                let body_bytes = to_bytes(req.into_body()).await.unwrap_or_default();
                let body_text =
                    std::string::String::from_utf8(body_bytes.to_vec()).unwrap_or_default();

                Ok(http::Response::builder()
                    .status(200)
                    .header("x-echo-method", method)
                    .header("x-echo-body", body_text)
                    .body(Body::from("ok"))
                    .expect("valid response"))
            })
        }
    }

    /// Never resolves - used to test that aborting a signal rejects the `fetch()` promise
    /// without needing a real, completing request race.
    struct PendingClient;

    impl LocalClient for PendingClient {
        fn send<'js, 'a>(
            &'a self,
            _ctx: &'a Ctx<'js>,
            _req: http::Request<JsBody<'js>>,
        ) -> LocalBoxFuture<'a, rquickjs::Result<http::Response<Body>>> {
            Box::pin(std::future::pending())
        }
    }

    /// Runs `body` as the contents of an `async` function, with `fetch`/`Request`/`Response`/
    /// `Headers`/`AbortController` and their transitive dependencies registered as globals, and
    /// `client` wired up as the `fetch()` backend. `body` is expected to throw on failure (e.g.
    /// via a plain `if (...) throw ...`).
    fn run(client: impl LocalClient + Send + 'static, body: &str) {
        futures::executor::block_on(async move {
            let rt = AsyncRuntime::new().unwrap();
            let ctx = AsyncContext::full(&rt).await.unwrap();

            ctx.async_with(async |ctx| {
                // `Headers`/`Request`/`Response` (via `TypedMultiMap`) go through
                // `BasePrimordials`, which needs the `$runtime` Core global that
                // `klaver_core::register` sets up - normally done by the `Environ`/`Vm` builder,
                // but this test drives a bare `Context`.
                klaver_core::register(&ctx)?;

                ctx.globals().set(
                    "EventTarget",
                    Class::<EventTarget>::create_constructor(&ctx)?,
                )?;
                EventTarget::add_event_target_prototype(&ctx)?;
                // `AbortController::abort()` constructs `Event::new_native` directly (not via
                // JS), but that instance still needs `Event`'s prototype accessors installed.
                Event::add_event_prototype(&ctx)?;

                AbortSignal::inherit(&ctx)?;
                ctx.globals().set(
                    "AbortSignal",
                    Class::<AbortSignal>::create_constructor(&ctx)?,
                )?;
                ctx.globals().set(
                    "AbortController",
                    Class::<AbortController>::create_constructor(&ctx)?,
                )?;

                ctx.globals().set(
                    DOMException::NAME,
                    Class::<DOMException>::create_constructor(&ctx)?,
                )?;
                DOMException::init(&ctx)?;

                ctx.globals()
                    .set(Headers::NAME, Class::<Headers>::create_constructor(&ctx)?)?;
                ctx.globals()
                    .set(Request::NAME, Class::<Request>::create_constructor(&ctx)?)?;
                ctx.globals().set(
                    Response::NAME,
                    Class::<Response>::create_constructor(&ctx)?,
                )?;

                ctx.globals().set("fetch", Func::from(Async(fetch)))?;

                set_local_client(&ctx, client)?;

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
    fn fetch_sends_the_specified_method_and_body() {
        run(
            EchoMethodClient,
            r#"
            const res = await fetch("http://example.com/", { method: "PUT", body: "hi" });
            if (res.headers.get("x-echo-method") !== "PUT") {
                throw new Error(`method was ${res.headers.get("x-echo-method")}`);
            }
            if (res.headers.get("x-echo-body") !== "hi") {
                throw new Error(`body was ${res.headers.get("x-echo-body")}`);
            }
        "#,
        );
    }

    #[test]
    fn fetch_returns_a_response_with_url_and_body() {
        run(
            EchoClient,
            r#"
            const res = await fetch("http://example.com/foo");
            if (res.status !== 201) throw new Error(`status was ${res.status}`);
            if (!res.ok) throw new Error("expected ok to be true");
            if (res.url !== "http://example.com/foo") throw new Error(`url was ${res.url}`);
            if (res.redirected !== false) throw new Error("expected redirected to be false");
            if (res.bodyUsed !== false) throw new Error("expected bodyUsed to be false before reading");

            const text = await res.text();
            if (text !== "hello") throw new Error(`text was ${text}`);
            if (res.bodyUsed !== true) throw new Error("expected bodyUsed to be true after reading");
        "#,
        );
    }

    #[test]
    fn fetch_with_a_request_and_init_applies_the_overrides() {
        run(
            EchoMethodClient,
            r#"
            const original = new Request("http://example.com/", { method: "POST", body: "original" });
            const res = await fetch(original, { method: "PUT", body: "overridden" });

            if (res.headers.get("x-echo-method") !== "PUT") {
                throw new Error(`method was ${res.headers.get("x-echo-method")}`);
            }
            if (res.headers.get("x-echo-body") !== "overridden") {
                throw new Error(`body was ${res.headers.get("x-echo-body")}`);
            }

            // `original` (the `input` passed to `fetch()`) must remain untouched.
            if (original.method !== "POST") throw new Error(`original method was ${original.method}`);
            const originalText = await original.text();
            if (originalText !== "original") throw new Error(`original text was ${originalText}`);
        "#,
        );
    }

    #[test]
    fn aborting_before_response_rejects_with_the_signals_reason() {
        run(
            PendingClient,
            r#"
            const controller = new AbortController();
            const promise = fetch("http://example.com/", { signal: controller.signal });
            controller.abort();

            let caught;
            try {
                await promise;
            } catch (err) {
                caught = err;
            }
            if (!caught) throw new Error("expected fetch to reject");
            if (caught.name !== "AbortError") throw new Error(`expected AbortError, got ${caught.name}`);
        "#,
        );
    }

    #[test]
    fn fetch_rejects_immediately_if_signal_already_aborted() {
        run(
            PendingClient,
            r#"
            const controller = new AbortController();
            controller.abort();

            let caught;
            try {
                await fetch("http://example.com/", { signal: controller.signal });
            } catch (err) {
                caught = err;
            }
            if (!caught) throw new Error("expected fetch to reject");
            if (caught.name !== "AbortError") throw new Error(`expected AbortError, got ${caught.name}`);
        "#,
        );
    }
}
