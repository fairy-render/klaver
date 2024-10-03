use std::{
    future::Future,
    pin::{pin, Pin},
};

use rquickjs::{AsyncContext, CatchResultExt, Ctx};

use crate::{timers::wait_timers, Error, Vm};

pub struct Context {
    ctx: AsyncContext,
}

impl Context {
    pub async fn new(vm: &Vm) -> Result<Context, Error> {
        let ctx = AsyncContext::full(&vm.rt).await?;
        Ok(Context { ctx })
    }
}

impl Context {
    pub async fn async_with<F, R>(&self, f: F) -> Result<R, Error>
    where
        F: for<'js> FnOnce(
                Ctx<'js>,
            )
                -> Pin<Box<dyn Future<Output = Result<R, Error>> + 'js + Send>>
            + Send,
        R: Send + 'static,
    {
        let timers = wait_timers(&self.ctx);
        let future = self.ctx.async_with(f);
        let mut future = pin!(future);

        tokio::select! {
            biased;
            ret = future.as_mut() => {
                return ret
            }
            ret = timers => {
                if let Err(err) = ret {
                    return Err(err)
                }
            }
        }

        future.await
    }

    pub async fn with<F, R>(&self, f: F) -> Result<R, Error>
    where
        F: for<'js> FnOnce(&Ctx<'js>) -> rquickjs::Result<R> + std::marker::Send,
        R: Send,
    {
        self.ctx
            .with(|ctx| f(&ctx).catch(&ctx).map_err(Into::into))
            .await
    }
}
