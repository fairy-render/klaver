use std::path::Path;

use klaver_core::CaugthException;
pub use klaver_core::RuntimeError;
use klaver_modules::Environ;

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
    let RuntimeError::Exception(CaugthException { stack, .. }) = &mut err else {
        return err;
    };

    let sourcemaps = env.modules().source_maps();
    for trace in stack {
        let Some((line, col)) =
            env.modules()
                .source_maps()
                .lookup(&trace.file, trace.line, trace.column)
        else {
            return err;
        };

        trace.line = line;
        trace.column = col;
    }

    err
}
