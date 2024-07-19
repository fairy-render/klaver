use bytes::Bytes;
use core::fmt;
use futures::TryStreamExt;
use klaver::{throw, throw_if};
use klaver_streams::{async_byte_iterator, AsyncByteIterError};
use reggie::Body;
// use reqwest::{Client, Response};
use reggie::http_body_util::BodyExt;
use rquickjs::{
    class::Trace, function::Opt, ArrayBuffer, Class, Ctx, Error, Exception, FromJs, IntoJs, Object,
    Value,
};
use tokio::sync::oneshot::Receiver;

use crate::{headers::Headers, module::Cancel};

#[derive(Trace, Clone, Copy)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    HEAD,
    OPTIONS,
    PATCH,
}

impl Method {
    pub fn as_str(&self) -> &'static str {
        match self {
            Method::GET => "GET",
            Method::POST => "POST",
            Method::PUT => "PUT",
            Method::DELETE => "DELETE",
            Method::HEAD => "HEAD",
            Method::OPTIONS => "OPTIONS",
            Method::PATCH => "PATCH",
        }
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<'js> IntoJs<'js> for Method {
    fn into_js(self, ctx: &rquickjs::prelude::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        let str = self.as_str();

        Ok(Value::from_string(rquickjs::String::from_str(
            ctx.clone(),
            str,
        )?))
    }
}

impl<'js> FromJs<'js> for Method {
    fn from_js(
        _ctx: &rquickjs::prelude::Ctx<'js>,
        value: rquickjs::Value<'js>,
    ) -> rquickjs::Result<Self> {
        let Some(method) = value.as_string() else {
            return Err(Error::new_from_js("value", "string"));
        };

        let method = match &*method.to_string()? {
            "GET" => Method::GET,
            _ => return Err(Error::new_from_js("string", "method")),
        };

        Ok(method)
    }
}

pub struct Options<'js> {
    cancel: Option<Class<'js, Cancel>>,
    method: Option<Method>,
    body: Option<ArrayBuffer<'js>>,
    headers: Option<Class<'js, Headers<'js>>>,
}

impl<'js> FromJs<'js> for Options<'js> {
    fn from_js(
        ctx: &rquickjs::prelude::Ctx<'js>,
        value: rquickjs::Value<'js>,
    ) -> rquickjs::Result<Self> {
        let Ok(obj) = value.try_into_object() else {
            return Err(Error::new_from_js("value", "object"));
        };

        let cancel = obj.get("cancel").ok();
        let method = obj.get("method").ok();
        let headers = obj.get("headers").ok();

        Ok(Options {
            cancel,
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
    url: rquickjs::String<'js>,
    #[qjs(get)]
    method: Method,
    #[qjs(get)]
    cancel: Option<Class<'js, Cancel>>,
    #[qjs(get)]
    headers: Class<'js, Headers<'js>>,
    body: Option<Body>,
}

impl<'js> Trace<'js> for Request<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.url.trace(tracer);
        self.method.trace(tracer);
        self.cancel.trace(tracer);
        self.headers.trace(tracer);
    }
}

impl<'js> Request<'js> {
    pub fn from_request<B: reggie::http_body::Body + Send + 'static>(
        ctx: &Ctx<'js>,
        request: reggie::http::Request<B>,
    ) -> rquickjs::Result<Class<'js, Request<'js>>>
    where
        B::Error: std::error::Error + Send + Sync + 'static,
        B::Data: Into<Bytes>,
    {
        let (parts, body) = request.into_parts();

        let url = rquickjs::String::from_str(ctx.clone(), &parts.uri.to_string())?;
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
                url,
                method,
                headers,
                cancel: None,
                body: Some(body),
            },
        )
    }

    pub async fn into_request(
        &mut self,
        ctx: Ctx<'js>,
    ) -> rquickjs::Result<(reggie::http::Request<Body>, Option<Receiver<()>>)> {
        let mut url = self.url.to_string()?;

        if url.starts_with("/") {
            url = format!("internal://{url}");
        }

        let mut builder = reggie::http::Request::builder()
            .method(self.method.as_str())
            .uri(url);

        for (k, vals) in self.headers.borrow().inner.iter() {
            for v in vals {
                builder = builder.header(k, v.to_string()?);
            }
        }

        let cancel = self.cancel.as_ref().and_then(|m| m.borrow_mut().create());

        let body = self
            .take_body(ctx.clone())
            .unwrap_or_else(|_| Body::empty());

        let body = throw_if!(ctx, builder.body(body));
        Ok((body, cancel))
    }

    fn take_body(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<Body> {
        let Some(body) = self.body.take() else {
            throw!(ctx, "body is exhausted")
        };

        Ok(body)
    }
}

#[rquickjs::methods]
impl<'js> Request<'js> {
    #[qjs(constructor)]
    pub fn new(
        ctx: Ctx<'js>,
        url: rquickjs::String<'js>,
        Opt(opts): Opt<Options<'js>>,
    ) -> rquickjs::Result<Self> {
        let (method, headers, cancel, body) = if let Some(opts) = opts {
            (opts.method, opts.headers, opts.cancel, opts.body)
        } else {
            (None, None, None, None)
        };

        let body = body
            .as_ref()
            .and_then(|m| m.as_bytes())
            .and_then(|m| Some(Body::from(m.to_vec())));

        Ok(Request {
            url,
            cancel,
            method: method.unwrap_or(Method::GET),
            headers: match headers {
                Some(ret) => ret,
                None => Class::instance(ctx.clone(), Headers::default())?,
            },
            body,
        })
    }

    pub async fn text(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<String> {
        let body = self.take_body(ctx.clone())?;

        match reggie::body::to_text(body).await {
            Ok(ret) => Ok(ret),
            Err(err) => Err(ctx.throw(Value::from_exception(Exception::from_message(
                ctx.clone(),
                &err.to_string(),
            )?))),
        }
    }

    pub async fn json(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        let body = self.take_body(ctx.clone())?;

        match reggie::body::to_json::<serde_json::Value, _>(body).await {
            Ok(ret) => Ok(crate::convert::from_json(ctx.clone(), ret)?),
            Err(err) => Err(ctx.throw(Value::from_exception(Exception::from_message(
                ctx.clone(),
                &err.to_string(),
            )?))),
        }
    }

    pub fn stream(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<Object<'js>> {
        let body = self.take_body(ctx.clone())?;

        let stream = reggie::body::to_stream(body);
        let stream = stream
            .map_ok(|m| m.to_vec())
            .map_err(|_| AsyncByteIterError);

        async_byte_iterator(ctx, stream)
    }
}
