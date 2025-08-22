use klaver_util::rquickjs::{self, Ctx, FromJs};

use crate::{
    context::Context,
    executor::{Execution, TaskExecutor, TaskHandle},
    resource::{Resource, ResourceKind},
};

#[derive(Clone)]
pub struct AsyncState {}

impl AsyncState {
    pub fn push<'js, T: Resource<'js> + 'js>(
        ctx: &Ctx<'js>,
        resource: T,
    ) -> rquickjs::Result<TaskHandle> {
        let executor = TaskExecutor::from_ctx(ctx)?;
        executor.push(ctx, resource)
    }

    pub async fn run_async<'js, T, R>(ctx: &Ctx<'js>, runner: T) -> rquickjs::Result<R>
    where
        T: AsyncFnOnce(Context<'js>) -> rquickjs::Result<R>,
        R: FromJs<'js>,
    {
        Self::run_async_with(
            ctx,
            Execution::default()
                .persist(true)
                .kind(ResourceKind::ROOT)
                .wait(true),
            runner,
        )
        .await
    }

    pub async fn run_async_with<'js, T, R>(
        ctx: &Ctx<'js>,
        execution: Execution,
        runner: T,
    ) -> rquickjs::Result<R>
    where
        T: AsyncFnOnce(Context<'js>) -> rquickjs::Result<R>,
        R: FromJs<'js>,
    {
        let executor = TaskExecutor::from_ctx(ctx)?;
        executor.run_async(ctx, execution, runner).await
    }

    pub fn run<'js, T, R>(ctx: &Ctx<'js>, runner: T) -> rquickjs::Result<R>
    where
        T: FnOnce(Context<'js>) -> rquickjs::Result<R>,
        R: FromJs<'js>,
    {
        Self::run_with(
            ctx,
            Execution::default()
                .persist(true)
                .kind(ResourceKind::ROOT)
                .wait(true),
            runner,
        )
    }

    pub fn run_with<'js, T, R>(
        ctx: &Ctx<'js>,
        execution: Execution,
        runner: T,
    ) -> rquickjs::Result<R>
    where
        T: FnOnce(Context<'js>) -> rquickjs::Result<R>,
        R: FromJs<'js>,
    {
        let executor = TaskExecutor::from_ctx(ctx)?;
        executor.run(ctx, execution, runner)
    }
}
