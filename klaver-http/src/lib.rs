use klaver_base::get_base;
use reggie::{Body, HttpClient, HttpClientFactory, SharedClientFactory};
use rquickjs::Ctx;

mod cancel;
pub mod client;
mod convert;
mod headers;
mod module;
mod request;
mod response;
// mod traits;

pub use self::request::Request;

pub type Module = module::js_http_mod;

pub fn register(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
    rquickjs::Module::declare_def::<module::js_http_mod, _>(ctx.clone(), "@klaver/http")?;
    Ok(())
}

pub struct Factory(pub SharedClientFactory);

pub fn set_client<T>(ctx: &Ctx<'_>, factory: T) -> rquickjs::Result<()>
where
    T: HttpClientFactory + Send + Sync + 'static,
    T::Client<reggie::Body>: Send + Sync,
    for<'a> <T::Client<reggie::Body> as HttpClient<reggie::Body>>::Future<'a>: Send,
    <T::Client<Body> as HttpClient<reggie::Body>>::Body: Into<Body>,
{
    let base = get_base(ctx)?;
    let mut base_mut = base.borrow_mut();

    if base_mut.extensions.contains::<Factory>() {
        base_mut.extensions.remove::<Factory>();
    }

    base_mut
        .extensions
        .insert(Factory(reggie::factory_arc::<T>(factory)));

    Ok(())
}

pub fn set_client_box(ctx: &Ctx<'_>, client: SharedClientFactory) -> rquickjs::Result<()> {
    let base = get_base(ctx)?;
    let mut base_mut = base.borrow_mut();

    if base_mut.extensions.contains::<Factory>() {
        base_mut.extensions.remove::<Factory>();
    }

    base_mut.extensions.insert(Factory(client));

    Ok(())
}

pub fn get_client(ctx: &Ctx<'_>) -> rquickjs::Result<reggie::Client> {
    let base = get_base(ctx)?;
    #[cfg(feature = "reqwest")]
    if !base.borrow().extensions.contains::<Factory>() {
        set_client(ctx, reggie::Reqwest::default())?;
    }

    let base_ref = base.borrow();

    if let Some(factory) = base_ref.extensions.get::<Factory>() {
        Ok(factory.0.create())
    } else {
        Err(rquickjs::Error::new_from_js_message(
            "null",
            "Client",
            "client factory not registered",
        ))
    }
}
