mod body_init;
mod body_state;
mod client;
mod convert;
mod facotry;
mod fetch;
mod headers;
mod method;
mod request;
mod response;
mod url;
mod url_search_params;

use klaver_shared::util::FunctionExt;
use rquickjs::{
    prelude::{Async, Func},
    Class, IntoJs,
};
use url_search_params::URLSearchParams;

pub use self::{
    client::Client, facotry::*, fetch::fetch, headers::Headers, request::Request,
    response::Response, url::Url,
};

pub fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
    decl.declare(stringify!(Response))?;
    decl.declare(stringify!(Request))?;
    decl.declare(stringify!(Headers))?;
    decl.declare(stringify!(URL))?;
    decl.declare(stringify!(Client))?;
    decl.declare(stringify!(fetch))?;
    decl.declare(stringify!(URLSearchParams))?;
    Ok(())
}

pub fn evaluate<'js>(
    ctx: &rquickjs::prelude::Ctx<'js>,
    exports: &rquickjs::module::Exports<'js>,
) -> rquickjs::Result<()> {
    export!(
        exports,
        ctx,
        Response,
        Request,
        Headers,
        Client,
        URLSearchParams
    );
    exports.export("URL", Class::<Url>::create_constructor(&ctx)?)?;

    let fetch = Func::new(Async(fetch))
        .into_js(&ctx)?
        .into_function()
        .unwrap();

    let client = Class::instance(ctx.clone(), Client::new(ctx.clone())?)?;
    let fetch = fetch.bind(ctx.clone(), (ctx.globals(), client))?;

    exports.export("fetch", fetch)?;
    Ok(())
}
