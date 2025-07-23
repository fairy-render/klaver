use rquickjs::{Class, Ctx, FromJs};

use crate::{request::Request, request_init::RequestInit};

pub enum FetchInit<'js> {
    Request(Class<'js, Request<'js>>),
}

impl<'js> FromJs<'js> for FetchInit<'js> {
    fn from_js(ctx: &Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        todo!()
    }
}

pub fn fetch<'js>(
    ctx: Ctx<'js>,
    url: FetchInit<'js>,
    req: RequestInit<'js>,
) -> rquickjs::Result<()> {
    todo!()
}
