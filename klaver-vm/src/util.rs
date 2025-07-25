use std::path::Path;

use klaver_modules::Environ;
use klaver_util::CaugthException;
pub use klaver_util::RuntimeError;

pub type Result<T> = core::result::Result<T, RuntimeError>;

#[allow(non_snake_case)]
pub const fn Ok<T>(value: T) -> Result<T> {
    Result::Ok(value)
}

// pub trait VmLike {
//     async fn with<F, R>(&self, f: F) -> Result<R>
//     where
//         F: for<'js> FnOnce(Ctx<'js>) -> Result<R> + std::marker::Send,
//         R: Send;

//     async fn async_with<F, R>(&self, f: F) -> Result<R>
//     where
//         F: for<'js> FnOnce(Ctx<'js>) -> BoxFuture<'js, Result<R>> + Send,
//         R: Send + 'static;

//     async fn run<T: Runnerable + 'static>(&self, task: T) -> Result<()>;
// }

pub(crate) fn update_locations(env: &Environ, mut err: RuntimeError) -> RuntimeError {
    if let Some(transform) = env.modules().transformer() {
        let RuntimeError::Exception(CaugthException { stack, .. }) = &mut err else {
            return err;
        };

        for trace in stack {
            let Some((line, col)) = transform.map(
                Path::new(&trace.file),
                trace.line as usize,
                trace.column as usize,
            ) else {
                continue;
            };

            trace.line = line as u32;
            trace.column = col as u32;
        }
    }

    err
}
