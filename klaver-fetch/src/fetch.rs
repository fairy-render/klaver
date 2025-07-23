use futures::{FutureExt, StreamExt};
use http::Uri;
use klaver_base::{AbortSignal, Emitter, EventKey};
use rquickjs::{Class, Coerced, Ctx, FromJs, String, prelude::Opt};
use rquickjs_util::{StringRef, throw, throw_if};

use crate::{
    Url, body::JsBody, client::Client, request::Request, request_init::RequestInit,
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
        client: &Client,
        init: Option<RequestInit<'js>>,
    ) -> rquickjs::Result<(
        http::Request<JsBody<'js>>,
        Option<Class<'js, AbortSignal<'js>>>,
    )> {
        match self {
            Self::Request(req) => req.borrow().to_native(ctx),
            Self::String(url) => {
                //
                let req = Request::new(ctx.clone(), Coerced(url), Opt(init))?;
                req.to_native(ctx)
            }
            Self::Url(_) => {
                todo!()
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

    let future = client.send(&ctx, req);

    if let Some(signal) = signal {
        let (sx, mut rx) = futures::channel::mpsc::channel(1);

        signal.borrow_mut().add_native_listener(
            EventKey::String(String::from_str(ctx.clone(), "abort")?),
            sx,
        );

        futures::select! {
            ret = future.fuse() => {

                match ret {
                    Ok(resp) => Response::from_native(&ctx, resp),
                    Err(err) => Err(err)
                }
            }
            _ = rx.next().fuse() => {
                throw!(ctx, "Aborted")
            }
        }
    } else {
        let resp = future.await?;
        Response::from_native(&ctx, resp)
    }
}
