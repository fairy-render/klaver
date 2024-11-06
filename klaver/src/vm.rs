use std::{future::Future, pin::Pin, task::Poll};

use futures::pin_mut;
use rquickjs::{
    context::EvalOptions, runtime::MemoryUsage, AsyncContext, AsyncRuntime, Ctx, FromJs,
};
use rquickjs_modules::Environ;
use rquickjs_util::RuntimeError;

use crate::Options;

pub struct Vm {
    context: AsyncContext,
    runtime: AsyncRuntime,
    env: Environ,
}

impl Vm {
    pub fn new() -> Options {
        Options::default()
    }

    pub async fn new_with(
        env: &Environ,
        max_stack_size: Option<usize>,
        max_mem: Option<usize>,
    ) -> Result<Vm, RuntimeError> {
        let runtime = AsyncRuntime::new()?;
        let context = AsyncContext::full(&runtime).await?;

        if let Some(ss) = max_stack_size {
            runtime.set_max_stack_size(ss).await;
        }

        if let Some(mm) = max_mem {
            runtime.set_memory_limit(mm).await;
        }

        env.init(&context).await?;

        Ok(Vm {
            runtime,
            context,
            env: env.clone(),
        })
    }

    pub fn env(&self) -> &Environ {
        &self.env
    }

    pub async fn run_gc(&self) {
        self.runtime.run_gc().await
    }

    pub async fn memory_usage(&self) -> MemoryUsage {
        self.runtime.memory_usage().await
    }

    pub async fn with<F, R>(&self, f: F) -> Result<R, RuntimeError>
    where
        F: for<'js> FnOnce(Ctx<'js>) -> Result<R, RuntimeError> + std::marker::Send,
        R: Send,
    {
        self.context.with(|ctx| f(ctx)).await
    }

    pub async fn run<S: Into<Vec<u8>> + Send, R>(
        &self,
        source: S,
        strict: bool,
    ) -> Result<R, RuntimeError>
    where
        for<'js> R: FromJs<'js>,
        R: Send,
    {
        self.with(|ctx| {
            let mut options = EvalOptions::default();
            options.strict = strict;
            options.promise = true;
            options.global = false;
            let val = ctx.eval_with_options::<R, _>(source, options)?;
            Ok(val)
        })
        .await
    }

    pub async fn async_with<F, R>(&self, f: F) -> Result<R, RuntimeError>
    where
        F: for<'js> FnOnce(
                Ctx<'js>,
            )
                -> Pin<Box<dyn Future<Output = Result<R, RuntimeError>> + 'js + Send>>
            + Send,
        R: Send + 'static,
    {
        klaver_wintercg::run(&self.context, f).await
    }

    pub fn idle(&self) -> Idle<'_> {
        Idle {
            inner: Box::pin(async move {
                let driver = self.runtime.drive();
                pin_mut!(driver);

                let timers = klaver_wintercg::wait_timers(&self.context);
                pin_mut!(timers);

                loop {
                    tokio::select! {
                      _ = driver.as_mut() => {
                        continue;
                      }
                      _ = timers.as_mut() => {
                        break;
                      }
                    }
                }

                Ok(())
            }),
        }
    }
}

pub struct Idle<'a> {
    #[cfg(feature = "parallel")]
    inner: Pin<Box<dyn Future<Output = Result<(), RuntimeError>> + Send + 'a>>,
    #[cfg(not(feature = "parallel"))]
    inner: Pin<Box<dyn Future<Output = Result<(), RuntimeError>> + 'a>>,
}

impl<'a> Future for Idle<'a> {
    type Output = Result<(), RuntimeError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        self.inner.as_mut().poll(cx)
    }
}
