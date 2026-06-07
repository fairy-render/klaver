use std::{any::Any, pin::Pin};

use klaver_core::RuntimeError;
use rquickjs::Ctx;

pub enum Request {
    WithAsync {
        callback: Box<
            dyn for<'js> FnOnce(
                    Ctx<'js>,
                ) -> Pin<
                    Box<dyn Future<Output = rquickjs::Result<Box<dyn Any + Send>>>>,
                > + Send,
        >,
    },
}

pub struct Worker {}

impl Worker {
    pub async fn async_with<F, R>(&self, f: F) -> Result<R, RuntimeError>
    where
        F: for<'js> FnOnce(Ctx<'js>) -> BoxFuture<'js, Result<R, RuntimeError>> + ParallelSend,
        R: ParallelSend + 'static,
    {
        self.context
            .async_with(|ctx| f(ctx))
            .await
            .map_err(|err| update_locations(&self.env, err))
    }
}
