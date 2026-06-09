#[macro_use]
mod macros;

mod base;
mod settings;

pub mod channel;

pub mod abort_controller;
#[cfg(feature = "streams")]
pub mod blob;
pub mod console;
#[cfg(feature = "crypto")]
pub mod crypto;
pub mod dom_exception;
pub mod encoding;
pub mod events;
#[cfg(feature = "fetch")]
pub mod fetch;
#[cfg(feature = "intl")]
pub mod intl;

mod backend;

pub mod streams;
#[cfg(feature = "timers")]
pub mod timers;

mod module;

pub use self::{
    backend::Backend,
    module::WinterTC,
    settings::{Settings, WinterTcInstance},
};

#[cfg(feature = "tokio")]
pub use self::backend::TokioBackend;

pub fn set_backend<T>(ctx: &rquickjs::Ctx<'_>, backend: T) -> rquickjs::Result<()>
where
    T: Backend + Send + Sync + 'static,
{
    let state = WinterTcInstance::from_ctx(ctx)?;
    state
        .borrow_mut()
        .set_backend(ctx, std::sync::Arc::new(backend))?;
    Ok(())
}
