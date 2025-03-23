use std::{future::Future, path::Path, pin::Pin, task::Poll};

use futures::{future::BoxFuture, pin_mut};
use rquickjs::{
    context::{self, EvalOptions},
    prelude::{Async, Func},
    runtime::MemoryUsage,
    AsyncContext, AsyncRuntime, Ctx, FromJs,
};
use rquickjs_modules::Environ;
use rquickjs_util::{throw, RuntimeError};

use crate::{
    realm::{create_realm, js_create_realm, JsRealm},
    Options,
};

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
        let runtime = env.create_runtime().await?;

        if let Some(ss) = max_stack_size {
            runtime.set_max_stack_size(ss).await;
        }

        if let Some(mm) = max_mem {
            runtime.set_memory_limit(mm).await;
        }

        let context = AsyncContext::full(&runtime).await?;

        env.init(&context).await?;

        let cloned_env = env.clone();
        let weak_runtime = runtime.weak();

        context
            .with(move |ctx| {
                ctx.store_userdata(JsRealm::new(weak_runtime, cloned_env))?;
                ctx.globals().set("createRealm", js_create_realm)?;

                Result::<_, rquickjs::Error>::Ok(())
            })
            .await?;

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

    pub async fn create_realm(&self) -> Result<Vm, RuntimeError> {
        let context = AsyncContext::full(&self.runtime).await?;
        self.env.init(&context).await?;
        Ok(Vm {
            context,
            runtime: self.runtime.clone(),
            env: self.env.clone(),
        })
    }

    pub async fn with<F, R>(&self, f: F) -> Result<R, RuntimeError>
    where
        F: for<'js> FnOnce(Ctx<'js>) -> Result<R, RuntimeError> + std::marker::Send,
        R: Send,
    {
        self.context
            .with(|ctx| f(ctx))
            .await
            .map_err(|err| update_locations(&self.env, err))
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
        .map_err(|err| update_locations(&self.env, err))
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
        klaver_wintercg::run(&self.context, f)
            .await
            .map_err(|err| update_locations(&self.env, err))
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

fn update_locations(env: &Environ, mut err: RuntimeError) -> RuntimeError {
    if let Some(transform) = env.modules().transformer() {
        let RuntimeError::Exception { stack, .. } = &mut err else {
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
