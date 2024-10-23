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

pub use self::{
    client::Client, facotry::*, fetch::fetch, headers::Headers, request::Request,
    response::Response, url::Url,
};

pub fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
    Ok(())
}

pub fn evaluate<'js>(
    ctx: &rquickjs::prelude::Ctx<'js>,
    exports: &rquickjs::module::Exports<'js>,
) -> rquickjs::Result<()> {
    Ok(())
}
