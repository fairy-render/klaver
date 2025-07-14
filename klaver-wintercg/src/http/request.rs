use bytes::Bytes;
use reggie::{http::Extensions, Body};
use rquickjs_util::{async_iterator::AsyncIterable, throw, throw_if, StringRef};
// use reqwest::{Client, Response};
use reggie::http_body_util::BodyExt;
use rquickjs::{
    class::Trace, function::Opt, ArrayBuffer, Class, Ctx, Error, Exception, FromJs, JsLifetime,
    Value,
};

use crate::{abort_controller::AbortSignal, streams::ReadableStream};

use super::{
    body_init::BodyInit, body_state::ResponseBodyKind, headers::HeadersInit, method::Method,
    url::StringOrUrl, Headers,
};

pub struct RequestInit<'js> {
    pub method: Option<Method>,
    pub body: Option<BodyInit<'js>>,
    pub signal: Option<Class<'js, AbortSignal<'js>>>,
    pub headers: Option<HeadersInit<'js>>,
}

impl<'js> FromJs<'js> for RequestInit<'js> {
    fn from_js(
        _ctx: &rquickjs::prelude::Ctx<'js>,
        value: rquickjs::Value<'js>,
    ) -> rquickjs::Result<Self> {
        if value.is_null() || value.is_undefined() {
            return Ok(RequestInit {
                method: None,
                body: None,
                signal: None,
                headers: None,
            });
        }

        let Ok(obj) = value.try_into_object() else {
            return Err(Error::new_from_js("value", "object"));
        };

        let signal = obj.get("signal")?;
        let method = obj.get("method")?;
        let headers = obj.get("headers")?;

        Ok(RequestInit {
            signal,
            method,
            headers,
            body: obj.get("body")?,
        })
    }
}

// #[derive(Trace)]
#[rquickjs::class]
pub struct Request<'js> {
    #[qjs(get)]
    url: String,
    #[qjs(get)]
    method: Method,
    #[qjs(get)]
    headers: Class<'js, Headers<'js>>,
    body: Option<ResponseBodyKind<'js>>,
    signal: Option<Class<'js, AbortSignal<'js>>>,
    extensions: Option<Extensions>,
}

unsafe impl<'js> JsLifetime<'js> for Request<'js> {
    type Changed<'to> = Request<'to>;
}

impl<'js> Trace<'js> for Request<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.url.trace(tracer);
        self.method.trace(tracer);
        self.signal.trace(tracer);
        self.headers.trace(tracer);
        self.body.trace(tracer);
    }
}

impl<'js> Request<'js> {
    pub fn from_request<B: reggie::http_body::Body + Sync + Send + 'static>(
        ctx: &Ctx<'js>,
        request: reggie::http::Request<B>,
    ) -> rquickjs::Result<Class<'js, Request<'js>>>
    where
        B::Error: std::error::Error + Send + Sync + 'static,
        B::Data: Into<Bytes>,
    {
        let (parts, body) = request.into_parts();

        let method = match parts.method {
            reggie::http::Method::GET => Method::GET,
            reggie::http::Method::POST => Method::POST,
            reggie::http::Method::PATCH => Method::PATCH,
            reggie::http::Method::DELETE => Method::DELETE,
            reggie::http::Method::PUT => Method::PUT,
            reggie::http::Method::OPTIONS => Method::OPTIONS,
            reggie::http::Method::HEAD => Method::HEAD,
            v => todo!("{v}"),
        };

        let headers = Headers::from_headers(ctx, parts.headers)?;

        let body = Body::from_streaming(body.map_err(|err| reggie::Error::conn(err)));

        Class::instance(
            ctx.clone(),
            Request {
                url: parts.uri.to_string(),
                method,
                signal: None,
                headers,
                body: Some(ResponseBodyKind::Body(Some(body))),
                extensions: Some(parts.extensions),
            },
        )
    }

    // pub async fn into_request(
    //     &mut self,
    //     ctx: Ctx<'js>,
    // ) -> rquickjs::Result<(reggie::http::Request<Body>, Option<Receiver<()>>)> {
    //     let url = self.url.to_string()?;

    //     let mut builder = reggie::http::Request::builder()
    //         .method(self.method.as_str())
    //         .uri(url);

    //     for (k, vals) in self.headers.borrow().inner.iter() {
    //         for v in vals {
    //             builder = builder.header(k, v.to_string()?);
    //         }
    //     }

    //     let cancel = self.cancel.as_ref().and_then(|m| m.borrow_mut().create());

    //     let body = self
    //         .take_body(ctx.clone())
    //         .unwrap_or_else(|_| Body::empty());

    //     let body = throw_if!(ctx, builder.body(body));
    //     Ok((body, cancel))
    // }

    // fn take_body(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<Body> {
    //     let Some(body) = self.body.take() else {
    //         throw!(ctx, "body is exhausted")
    //     };

    //     Ok(body)
    // }

    pub async fn into_request(
        &mut self,
        ctx: Ctx<'js>,
    ) -> rquickjs::Result<(
        reggie::http::Request<Body>,
        Option<tokio::sync::oneshot::Receiver<()>>,
    )> {
        let mut builder = reggie::http::Request::builder()
            .method(self.method.as_str())
            .uri(&self.url);

        for (k, vals) in self.headers.borrow().inner.iter() {
            for v in vals {
                builder = builder.header(k, v.to_string()?);
            }
        }

        // let cancel = self.cancel.as_ref().and_then(|m| m.borrow_mut().create());

        let cancel = if let Some(signal) = &self.signal {
            Some(signal.borrow_mut().channel(ctx.clone())?)
        } else {
            None
        };

        let body = if let Some(body) = self.body.as_mut() {
            body.to_reggie(ctx.clone()).await?
        } else {
            Body::empty()
        };

        let mut body = throw_if!(ctx, builder.body(body));
        if let Some(ext) = self.extensions.take() {
            *body.extensions_mut() = ext;
        }
        Ok((body, cancel))
    }

    // fn take_body(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<Body> {
    //     let Some(body) = self.body.take() else {
    //         throw!(ctx, "body is exhausted")
    //     };

    //     Ok(body)
    // }
}

#[rquickjs::methods]
impl<'js> Request<'js> {
    #[qjs(constructor)]
    pub fn new(
        ctx: Ctx<'js>,
        url: StringOrUrl<'js>,
        Opt(opts): Opt<RequestInit<'js>>,
    ) -> rquickjs::Result<Self> {
        let (method, headers, signal, body) = if let Some(opts) = opts {
            (opts.method, opts.headers, opts.signal, opts.body)
        } else {
            (None, None, None, None)
        };

        let body = if let Some(body) = body {
            let body = ResponseBodyKind::Stream(body.to_stream(&ctx)?);
            Some(body)
        } else {
            None
        };

        Ok(Request {
            url: url.as_str()?,
            signal,
            method: method.unwrap_or(Method::GET),
            headers: match headers {
                Some(ret) => ret.inner,
                None => Class::instance(ctx.clone(), Headers::default())?,
            },
            body,
            extensions: None,
        })
    }

    pub async fn text(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<String> {
        let body = if let Some(body) = self.body.as_mut() {
            body.bytes(ctx.clone()).await?
        } else {
            throw!(ctx, "Response has no body")
        };

        match String::from_utf8(body) {
            Ok(ret) => Ok(ret),
            Err(err) => Err(ctx.throw(Value::from_exception(Exception::from_message(
                ctx.clone(),
                &err.to_string(),
            )?))),
        }
    }

    #[qjs(rename = "arrayBuffer")]
    pub async fn array_buffer(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<ArrayBuffer<'js>> {
        let body = if let Some(body) = self.body.as_mut() {
            body.bytes(ctx.clone()).await?
        } else {
            throw!(ctx, "Response has no body")
        };

        ArrayBuffer::new(ctx, body)
    }

    pub async fn json(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        let body = if let Some(body) = self.body.as_mut() {
            body.bytes(ctx.clone()).await?
        } else {
            throw!(ctx, "Response has no body")
        };

        match serde_json::from_slice(&body) {
            Ok(ret) => Ok(super::convert::from_json(ctx.clone(), ret)?),
            Err(err) => Err(ctx.throw(Value::from_exception(Exception::from_message(
                ctx.clone(),
                &err.to_string(),
            )?))),
        }
    }

    pub async fn form_data(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        let Some(content_type) = self
            .headers
            .borrow()
            .inner
            .get("Content-Type")
            .and_then(|m| m.last())
            .cloned()
        else {
            throw!(ctx, "Not multipart")
        };

        let Ok(boundary) = multer::parse_boundary(StringRef::from_string(content_type)?.as_str())
        else {
            throw!(ctx, "Not multipart")
        };

        let body = if let Some(body) = self.body.as_mut() {
            body.bytes(ctx.clone()).await?
        } else {
            throw!(ctx, "Response has no body")
        };

        let mut multipart = multer::Multipart::new(
            futures::stream::once(async move { rquickjs::Result::Ok(body) }),
            boundary,
        );

        loop {
            let Some(next) = throw_if!(ctx, multipart.next_field().await) else {
                break;
            };
        }

        todo!()
    }

    #[qjs(get, rename = "bodyUsed")]
    pub fn body_used(&self) -> rquickjs::Result<bool> {
        Ok(self.body.as_ref().map(|m| m.is_consumed()).unwrap_or(true))
    }

    #[qjs(get)]
    pub fn body(
        &mut self,
        ctx: Ctx<'js>,
    ) -> rquickjs::Result<Option<Class<'js, ReadableStream<'js>>>> {
        if let Some(body) = self.body.as_mut() {
            Ok(Some(body.stream(ctx)?))
        } else {
            Ok(None)
        }
    }
}
