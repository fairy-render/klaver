use rquickjs::{prelude::Opt, Class, Ctx, FromJs};

use super::{request::RequestInit, url::StringOrUrl, Client, Request, Response, Url};

pub enum FetchResource<'js> {
    String(rquickjs::String<'js>),
    Url(Class<'js, Url>),
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
    client: Class<'js, Client>,
    req: FetchResource<'js>,
    init: Opt<RequestInit<'js>>,
) -> rquickjs::Result<Class<'js, Response<'js>>> {
    let req = req.into_request(ctx.clone(), init).await?;
    client.borrow().send(ctx, req).await
}
