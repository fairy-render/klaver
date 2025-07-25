use futures::future::{BoxFuture, LocalBoxFuture};
use klaver_modules::Environ;
use klaver_runner::{Runner, Runnerable};
use klaver_util::RuntimeError;
use rquickjs::{
    AsyncContext, AsyncRuntime, CatchResultExt, Ctx, Function, Module, Object, Value,
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

    pub async fn run<T: Runnerable + 'static>(&self, task: T) -> Result<(), RuntimeError> {
        Runner::new(&self.context, task)
            .run()
            .await
            .map_err(|err| update_locations(&self.env, err))
    }

    pub async fn idle(&self) {
        self.runtime().idle().await;
    }

    pub async fn run_module<A: 'static>(&self, module: &str, args: A) -> Result<(), RuntimeError>
    where
        A: for<'js> IntoArgs<'js>,
    {
        self.run(ModuleRunner {
            module: module.to_string(),
            args,
            func: Default::default(),
        })
        .await?;

        Ok(())
    }
}

struct ModuleRunner<A> {
    module: String,
    args: A,
    func: Option<String>,
}

impl<A> Runnerable for ModuleRunner<A>
where
    A: for<'js> IntoArgs<'js>,
    A: 'static,
{
    type Future<'js> = LocalBoxFuture<'js, Result<(), RuntimeError>>;

    fn call<'js>(self, ctx: Ctx<'js>, _worker: klaver_runner::Workers) -> Self::Future<'js> {
        Box::pin(async move {
            let promise = Module::import(&ctx, self.module).catch(&ctx)?;
            let module = promise.into_future::<Object>().await.catch(&ctx)?;

            if let Some(fun) = self.func {
                let func = module.get::<_, Function>(&fun).catch(&ctx)?;

                let ret = func.call::<_, Value>(self.args).catch(&ctx)?;
                if let Some(future) = ret.into_promise() {
                    future.into_future::<()>().await?;
                }
            }

            Ok(())
        })
    }
}
