mod body;
mod body_init;
mod client;
mod fetch;
mod headers;
mod method;
mod module;
mod request;
mod request_init;
mod response;
mod url;
mod url_search_params;

pub use self::{
    body::*, client::*, headers::Headers, method::Method, module::FetchModule, response::Response,
    url::Url, url_search_params::URLSearchParams,
};

pub use reggie::Body;

#[cfg(feature = "reqwest")]
pub use reqwest;
use rquickjs::Ctx;

pub fn set_shared_client<T>(ctx: &Ctx<'_>, client: T) -> rquickjs::Result<()>
where
    T: SharedClient + 'static,
{
    let state = Client::from_ctx(ctx)?;
    state.set_shared_client(client);
    Ok(())
}

pub fn set_local_client<T>(ctx: &Ctx<'_>, client: T) -> rquickjs::Result<()>
where
    T: LocalClient + 'static,
{
    let state = Client::from_ctx(ctx)?;
    state.set_local_client(client);
    Ok(())
}
