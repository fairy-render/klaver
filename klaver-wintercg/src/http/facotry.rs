use klaver::core::get_core;
use reggie::{Body, HttpClient, HttpClientFactory, SharedClientFactory};
use rquickjs::Ctx;

struct Factory(pub SharedClientFactory);

pub fn set_http_client<T>(ctx: &Ctx<'_>, factory: T) -> rquickjs::Result<()>
where
    T: HttpClientFactory + Send + Sync + 'static,
    T::Client<reggie::Body>: Send + Sync,
    for<'a> <T::Client<reggie::Body> as HttpClient<reggie::Body>>::Future<'a>: Send,
    <T::Client<Body> as HttpClient<reggie::Body>>::Body: Into<Body>,
{
    set_http_client_box(ctx, reggie::factory_arc::<T>(factory))
}

pub fn set_http_client_box(ctx: &Ctx<'_>, client: SharedClientFactory) -> rquickjs::Result<()> {
    let base = get_core(ctx)?;
    let mut base_mut = base.borrow_mut();

    if base_mut.extensions().contains::<Factory>() {
        base_mut.extensions_mut().remove::<Factory>();
    }

    base_mut.extensions_mut().insert(Factory(client));

    Ok(())
}

pub fn get_http_client(ctx: &Ctx<'_>) -> rquickjs::Result<reggie::Client> {
    let base = get_core(ctx)?;
    #[cfg(feature = "reqwest")]
    if !base.borrow().extensions().contains::<Factory>() {
        set_http_client(ctx, reggie::Reqwest::default())?;
    }

    let base_ref = base.borrow();

    if let Some(factory) = base_ref.extensions().get::<Factory>() {
        Ok(factory.0.create())
    } else {
        Err(rquickjs::Error::new_from_js_message(
            "null",
            "Client",
            "client factory not registered",
        ))
    }
}
