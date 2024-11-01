#[macro_use]
mod macros;

pub mod abort_controller;
mod blob;
mod config;
pub mod console;
pub mod dom_exception;
pub mod event_target;
mod global;
mod module;
mod multimap;
pub mod streams;
mod timers;

#[cfg(feature = "crypto")]
pub mod crypto;
#[cfg(feature = "encoding")]
pub mod encoding;
mod error;
#[cfg(feature = "http")]
pub mod http;
pub mod performance;

use std::{future::Future, pin::Pin};

pub use module::Module;
use rquickjs::{AsyncContext, Ctx};

pub use self::{
    dom_exception::DOMException, error::RuntimeError, event_target as events, global::*,
    timers::wait_timers,
};

pub async fn run<F, R>(context: &AsyncContext, f: F) -> Result<R, RuntimeError>
where
    F: for<'js> FnOnce(
            Ctx<'js>,
        )
            -> Pin<Box<dyn Future<Output = Result<R, RuntimeError>> + 'js + Send>>
        + Send,
    R: Send + 'static,
{
    let timers = wait_timers(context);
    let future = context.async_with(f);
    tokio::pin!(future);

    tokio::select! {
        biased;
        ret = future.as_mut() => {
            return ret
        }
        ret = timers => {
            if let Err(err) = ret {
                return Err(err.into())
            }
        }
    }

    future.await
}
