mod body_init;
mod body_state;
mod convert;
mod fetch;
mod form_data;
mod headers;
mod method;
mod request;
mod response;
mod url;
mod url_search_params;

use rquickjs::{
    prelude::{Async, Func},
    Class, IntoJs,
};
use rquickjs_util::{iterator::Iterable, util::FunctionExt};
use url_search_params::URLSearchParams;

use crate::config::WinterCG;

pub use self::{
    fetch::fetch, form_data::FormData, headers::Headers, request::Request, response::Response,
    url::Url,
};

pub fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
    decl.declare(stringify!(Response))?;
    decl.declare(stringify!(Request))?;
    decl.declare(stringify!(Headers))?;
    decl.declare(stringify!(URL))?;
    decl.declare(stringify!(fetch))?;
    decl.declare(stringify!(URLSearchParams))?;
    Ok(())
}

pub fn register<'js>(
    ctx: &rquickjs::prelude::Ctx<'js>,
    winter: &Class<'js, WinterCG<'js>>,
) -> rquickjs::Result<()> {
    define!(ctx, Response, Request, Headers, URLSearchParams, FormData);
    URLSearchParams::add_iterable_prototype(ctx)?;

    ctx.globals()
        .set("URL", Class::<Url>::create_constructor(&ctx)?)?;

    let fetch = Func::new(Async(fetch))
        .into_js(&ctx)?
        .into_function()
        .unwrap();

    let fetch = fetch.bind(ctx.clone(), (ctx.globals(), winter.clone()))?;
    ctx.globals().set("fetch", fetch)?;

    Ok(())
}
