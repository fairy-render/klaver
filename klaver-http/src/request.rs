use core::fmt;

use reggie::Body;
// use reqwest::{Client, Response};
use rquickjs::{class::Trace, function::Opt, Class, Ctx, Error, FromJs, IntoJs, Value};
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
    body: (),
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
            body: (),
        })
    }
}

#[derive(Trace)]
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
}

impl<'js> Request<'js> {
    // pub async fn exec(&self, ctx: Ctx<'js>, client: &Client) -> rquickjs::Result<Response> {
    //     let url = self.url.to_string()?;

    //     let mut builder = reggie::http::Request::builder();
    //     let mut builder = match self.method {
    //         Method::GET => builder.method("GET"),
    //         Method::POST => builder.me,
    //         Method::PUT => client.put(url),
    //         Method::DELETE => client.delete(url),
    //         Method::HEAD => client.head(url),
    //         Method::OPTION => client.request(reqwest::Method::OPTIONS, url),
    //     };

    //     for (k, vals) in self.headers.borrow().inner.iter() {
    //         for v in vals {
    //             builder = builder.header(k, v.to_string()?);
    //         }
    //     }

    //     let run = || async {
    //         let resp = match builder.send().await {
    //             Ok(ret) => ret,
    //             Err(err) => {
    //                 return Err(ctx.throw(Value::from_exception(Exception::from_message(
    //                     ctx.clone(),
    //                     &err.to_string(),
    //                 )?)))
    //             }
    //         };

    //         Ok(resp)
    //     };
    //     if let Some(mut cancel) = self.cancel.as_ref().and_then(|m| m.borrow_mut().create()) {
    //         tokio::select! {
    //             _ = &mut cancel => {
    //                  Err(ctx.throw(Value::from_exception(Exception::from_message(
    //                     ctx.clone(),
    //                     "CANCEL",
    //                 )?)))
    //             }
    //             resp = run() => {
    //                  resp
    //             }
    //         }
    //     } else {
    //         run().await
    //     }
    // }

    pub fn from_request<B>(
        ctx: &Ctx<'js>,
        request: reggie::http::Request<B>,
    ) -> rquickjs::Result<Class<'js, Request<'js>>> {
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

        Class::instance(
            ctx.clone(),
            Request {
                url,
                method,
                headers,
                cancel: None,
            },
        )
    }

    pub async fn into_request(
        &self,
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

        Ok((builder.body(Body::empty()).unwrap(), cancel))
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
        let (method, headers, cancel) = if let Some(opts) = opts {
            (opts.method, opts.headers, opts.cancel)
        } else {
            (None, None, None)
        };

        Ok(Request {
            url,
            cancel,
            method: method.unwrap_or(Method::GET),
            headers: match headers {
                Some(ret) => ret,
                None => Class::instance(ctx.clone(), Headers::default())?,
            },
        })
    }
}
