use std::marker::PhantomData;

use futures::future::BoxFuture;
use klaver_modules::Environ;
use klaver_task::{EventLoop, Runner};
use klaver_util::RuntimeError;
use rquickjs::{
    AsyncContext, AsyncRuntime, Ctx, FromJs, Function, Module, Object, Value,
    markers::ParallelSend, prelude::IntoArgs,
};

use crate::update_locations;

pub struct Context {
    pub(crate) context: AsyncContext,
    pub(crate) env: Environ,
}

impl Context {
    pub fn env(&self) -> &Environ {
        &self.env
    }

    pub fn runtime(&self) -> &AsyncRuntime {
        self.context.runtime()
    }

    pub async fn with<F, R>(&self, f: F) -> Result<R, RuntimeError>
    where
        F: for<'js> FnOnce(Ctx<'js>) -> Result<R, RuntimeError> + ParallelSend,
        R: ParallelSend,
    {
        self.context
            .with(|ctx| f(ctx))
            .await
            .map_err(|err| update_locations(&self.env, err))
    }

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

    pub async fn run<T, R>(&self, task: T) -> Result<R, RuntimeError>
    where
        T: for<'js> Runner<'js, Output = R>,
        R: 'static,
    {
        EventLoop::new(task)
            .run(&self.context)
            .await
            .map_err(|err| update_locations(&self.env, err))
    }

    pub async fn idle(&self) {
        self.runtime().idle().await;
    }

    pub async fn run_module(&self, module: &str) -> Result<(), RuntimeError> {
        self.run(ModuleRunner {
            module: module.to_string(),
        })
        .await?;

        Ok(())
    }

    pub async fn call_export<A, R: 'static>(
        &self,
        module: &str,
        export: &str,
        args: A,
    ) -> Result<R, RuntimeError>
    where
        A: for<'js> IntoArgs<'js>,
        R: for<'js> FromJs<'js>,
    {
        let export = CallExport::<A, R> {
            module: module.to_string(),
            export: export.to_string(),
            args,
            ret: PhantomData,
        };

        self.run(export).await
    }
}

struct ModuleRunner {
    module: String,
}

impl<'js> Runner<'js> for ModuleRunner {
    type Output = ();

    async fn run(self, ctx: klaver_task::TaskCtx<'js>) -> rquickjs::Result<Self::Output> {
        let promise = Module::import(&ctx, self.module)?;
        let _ = promise.into_future::<()>().await?;

        Ok(())
    }
}

struct CallExport<A, R> {
    module: String,
    args: A,
    export: String,
    ret: PhantomData<fn() -> R>,
}

impl<'js, A, R> Runner<'js> for CallExport<A, R>
where
    A: IntoArgs<'js>,
    R: FromJs<'js>,
{
    type Output = R;

    async fn run(self, ctx: klaver_task::TaskCtx<'js>) -> rquickjs::Result<Self::Output> {
        let promise = Module::import(&ctx, self.module)?;
        let module = promise.into_future::<Object>().await?;

        let func = module.get::<_, Function>(&self.export)?;

        let mut ret = func.call::<_, Value>(self.args)?;
        if let Some(future) = ret.clone().into_promise() {
            ret = future.into_future().await?;
        }

        R::from_js(&ctx, ret)
    }
}
