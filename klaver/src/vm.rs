use std::{future::Future, pin::Pin};

use rquickjs::{
    context::EvalOptions, runtime::MemoryUsage, AsyncContext, AsyncRuntime, Ctx, FromJs,
};
use rquickjs_modules::Environ;
use rquickjs_util::RuntimeError;

use crate::Options;

pub struct Vm {
    context: AsyncContext,
    runtime: AsyncRuntime,
}

impl Vm {
    pub fn new() -> Options {
        Options::default()
    }

    pub(super) async fn new_with(
        env: &Environ,
        max_stack_size: Option<usize>,
        max_mem: Option<usize>,
    ) -> Result<Vm, RuntimeError> {
        let runtime = AsyncRuntime::new()?;
        let context = AsyncContext::full(&runtime).await?;

        env.init(&context).await?;

        Ok(Vm { runtime, context })
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
        rquickjs_wintercg::run(&self.context, f).await
    }
}
