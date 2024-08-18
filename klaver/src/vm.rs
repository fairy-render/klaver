use std::{
    future::Future,
    path::PathBuf,
    pin::{pin, Pin},
    task::Poll,
};

use rquickjs::{context::EvalOptions, AsyncContext, AsyncRuntime, CatchResultExt, Ctx, FromJs};

use crate::{
    base::{
        init as init_base,
        timers::{poll_timers, process_timers},
    },
    error::Error,
    modules::ModuleInfo,
};

#[derive(Default, Clone)]
pub struct VmOptions {
    modules: crate::modules::Modules,
    max_stack_size: Option<usize>,
    memory_limit: Option<usize>,
}

impl VmOptions {
    pub fn modules(&mut self) -> &mut crate::modules::Modules {
        &mut self.modules
    }

    pub fn module<T: ModuleInfo>(mut self) -> Self {
        self.modules.register_module::<T>();
        self
    }

    pub fn search_path(mut self, search_path: impl Into<PathBuf>) -> Self {
        self.modules.add_search_path(search_path);
        self
    }

    pub async fn build(self) -> Result<Vm, Error> {
        Vm::new(self).await
    }
}
pub struct Vm {
    ctx: AsyncContext,
    rt: AsyncRuntime,
}

impl Vm {
    pub async fn new(options: VmOptions) -> Result<Vm, Error> {
        let rt = AsyncRuntime::new()?;

        if let Some(stack) = options.max_stack_size {
            rt.set_max_stack_size(stack).await;
        }

        if let Some(memory) = options.memory_limit {
            rt.set_memory_limit(memory).await;
        }

        let ctx = AsyncContext::full(&rt).await?;

        ctx.with(init_base).await?;

        options.modules.attach(&rt, &ctx).await?;

        Ok(Vm { ctx, rt })
    }

    pub async fn run_with<F, R>(&self, f: F) -> Result<R, Error>
    where
        F: for<'js> FnOnce(&Ctx<'js>) -> rquickjs::Result<R> + std::marker::Send,
        R: Send,
    {
        self.ctx
            .with(|ctx| f(&ctx).catch(&ctx).map_err(Into::into))
            .await
    }

    pub async fn run<S: Into<Vec<u8>> + Send, R>(&self, source: S, strict: bool) -> Result<R, Error>
    where
        for<'js> R: FromJs<'js>,
        R: Send,
    {
        self.run_with(|ctx| {
            let mut options = EvalOptions::default();
            options.strict = strict;
            options.promise = true;
            options.global = false;
            let val = ctx.eval_with_options::<R, _>(source, options)?;
            Ok(val)
        })
        .await
    }

    pub async fn async_with<F, R>(&self, f: F) -> Result<R, Error>
    where
        F: for<'js> FnOnce(
                Ctx<'js>,
            )
                -> Pin<Box<dyn Future<Output = Result<R, Error>> + 'js + Send>>
            + Send,
        R: Send + 'static,
    {
        let idle = self.idle();
        let future = self.ctx.async_with(f);
        let mut future = pin!(future);

        tokio::select! {
            biased;
            ret = future.as_mut() => {
                return ret
            }
            ret = idle => {
                if let Err(err) = ret {
                    return Err(err)
                }
            }
        }

        future.await
    }

    pub fn idle(&self) -> Idle<'_> {
        Idle {
            inner: Box::pin(async move {
                let mut i = 0;
                loop {
                    let has_timers = self.ctx.with(|ctx| process_timers(&ctx)).await?;

                    if !has_timers && i > 0 {
                        break;
                    }

                    let sleep = self.ctx.with(|ctx| poll_timers(&ctx)).await?;

                    // if !has_timers {
                    //     self.rt.execute_pending_job().await.ok();
                    // }

                    sleep.await;

                    i += 1;
                }

                Ok(())
            }),
        }
    }
}

pub struct Idle<'a> {
    inner: Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>>,
}

impl<'a> Future for Idle<'a> {
    type Output = Result<(), Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        self.inner.as_mut().poll(cx)
    }
}
