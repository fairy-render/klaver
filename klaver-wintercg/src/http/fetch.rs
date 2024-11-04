use rquickjs::{prelude::Opt, Class, Ctx, Exception, FromJs, Value};

use crate::config::WinterCG;

use super::{request::RequestInit, url::StringOrUrl, Request, Response, Url};

pub enum FetchResource<'js> {
    String(rquickjs::String<'js>),
    Url(Class<'js, Url<'js>>),
    Request(Class<'js, Request<'js>>),
}

impl<'js> FetchResource<'js> {
    async fn into_request(
        self,
        ctx: Ctx<'js>,
        init: Opt<RequestInit<'js>>,
    ) -> rquickjs::Result<Class<'js, Request<'js>>> {
        match self {
            Self::String(s) => Class::instance(
                ctx.clone(),
                Request::new(ctx.clone(), StringOrUrl::String(s), init)?,
            ),
            Self::Url(url) => Class::instance(
                ctx.clone(),
                Request::new(ctx.clone(), StringOrUrl::Url(url), init)?,
            ),
            Self::Request(req) => Ok(req),
        }
    }
}

impl<'js> FromJs<'js> for FetchResource<'js> {
    fn from_js(ctx: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        if value.is_string() {
            Ok(FetchResource::String(rquickjs::String::from_js(
                ctx, value,
            )?))
        } else if let Ok(ret) = Class::<Url>::from_js(ctx, value.clone()) {
            Ok(FetchResource::Url(ret.clone()))
        } else if let Ok(ret) = Class::<Request>::from_js(ctx, value.clone()) {
            Ok(FetchResource::Request(ret.clone()))
        } else {
            Err(rquickjs::Error::new_from_js(value.type_name(), "reqsource"))
        }
    }
}

#[rquickjs::function]
pub async fn fetch<'js>(
    ctx: Ctx<'js>,
    env: Class<'js, WinterCG<'js>>,
    request: FetchResource<'js>,
    init: Opt<RequestInit<'js>>,
) -> rquickjs::Result<Class<'js, Response<'js>>> {
    let client = env.borrow().http_client().clone();

    let req = request.into_request(ctx.clone(), init).await?;

    let mut req = req.borrow_mut();
    let (req, cancel) = req.into_request(ctx.clone()).await?;

    let url = req.uri().clone();

    let run = || async {
        let resp = match client.request(req).await {
            Ok(ret) => ret,
            Err(err) => {
                return Err(ctx.throw(Value::from_exception(Exception::from_message(
                    ctx.clone(),
                    &err.to_string(),
                )?)));
            }
        };

        Ok(resp)
    };

    let resp = if let Some(mut cancel) = cancel {
        tokio::select! {
            _ = &mut cancel => {
                 return Err(ctx.throw(Value::from_exception(Exception::from_message(
                    ctx.clone(),
                    "CANCEL",
                )?)))
            }
            resp = run() => {
                 resp?
            }
        }
    } else {
        run().await?
    };

    Class::instance(
        ctx.clone(),
        Response::from_response(ctx.clone(), &url.to_string(), resp)?,
    )
}
