use std::{
    borrow::Cow,
    future::Future,
    path::{Path, PathBuf},
    pin::{pin, Pin},
    task::Poll,
};

use rquickjs::{
    context::EvalOptions, runtime::MemoryUsage, AsyncContext, AsyncRuntime, CatchResultExt, Ctx,
    FromJs,
};

use crate::{
    base::{
        init as init_base,
        timers::{poll_timers, process_timers},
    },
    context::Context,
    error::Error,
    modules::{Builder, ModuleInfo, Modules},
    timers::wait_timers,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleTypings {
    pub name: &'static str,
    pub typings: Cow<'static, str>,
}

#[derive(Default)]
pub struct VmOptions {
    pub(crate) modules: crate::modules::ModulesBuilder,
    pub(crate) max_stack_size: Option<usize>,
    pub(crate) memory_limit: Option<usize>,
    pub(crate) typings: Vec<ModuleTypings>,
}

impl VmOptions {
    pub fn modules(&mut self) -> &mut crate::modules::ModulesBuilder {
        &mut self.modules
    }

    pub fn module<T: ModuleInfo>(mut self) -> Self {
        T::register(&mut Builder::new(&mut self.modules, &mut self.typings));

        if let Some(typings) = T::typings() {
            self.typings.push(ModuleTypings {
                name: T::NAME,
                typings,
            });
        }

        self
    }

    pub fn search_path(mut self, search_path: impl Into<PathBuf>) -> Self {
        self.modules.add_search_path(search_path);
        self
    }

    pub fn typings(&self) -> &[ModuleTypings] {
        &self.typings
    }

    pub async fn build(self) -> Result<Vm, Error> {
        Vm::new(self).await
    }
}
pub struct Vm {
    ctx: AsyncContext,
    pub(crate) rt: AsyncRuntime,
    modules: Modules,
}

impl Vm {
    pub async fn new_with(
        modules: Modules,
        max_stack_size: Option<usize>,
        memory_limit: Option<usize>,
    ) -> Result<Vm, Error> {
        let rt = AsyncRuntime::new()?;

        if let Some(stack) = max_stack_size {
            rt.set_max_stack_size(stack).await;
        }

        if let Some(memory) = memory_limit {
            rt.set_memory_limit(memory).await;
        }

        let ctx = AsyncContext::full(&rt).await?;

        ctx.with(init_base).await?;
        modules.attach(&rt).await?;

        Ok(Vm { ctx, rt, modules })
    }

    pub async fn new(options: VmOptions) -> Result<Vm, Error> {
        Self::new_with(
            options.modules.build()?,
            options.max_stack_size,
            options.memory_limit,
        )
        .await
    }

    pub async fn run_gc(&self) {
        self.rt.run_gc().await
    }

    pub async fn memory_usage(&self) -> MemoryUsage {
        self.rt.memory_usage().await
    }

    pub async fn with<F, R>(&self, f: F) -> Result<R, Error>
    where
        F: for<'js> FnOnce(Ctx<'js>) -> Result<R, Error> + std::marker::Send,
        R: Send,
    {
        self.ctx.with(|ctx| f(ctx)).await
    }

    pub async fn run<S: Into<Vec<u8>> + Send, R>(&self, source: S, strict: bool) -> Result<R, Error>
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

    pub async fn create_context(&self) -> Result<Context, Error> {
        Context::new(self).await
    }

    pub fn idle(&self) -> Idle<'_> {
        Idle {
            inner: Box::pin(async move {
                let mut driver = self.rt.drive();
                let mut driver = pin!(driver);
                let mut timers = pin!(wait_timers(&self.ctx));

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
    inner: Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>>,
}

impl<'a> Future for Idle<'a> {
    type Output = Result<(), Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        self.inner.as_mut().poll(cx)
    }
}
