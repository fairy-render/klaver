use klaver_util::rquickjs::{self, Ctx, FromJs};

use crate::{context::Context, executor::TaskExecutor, resource::ResourceKind, runtime::Runtime};

pub struct EventLoop {}

impl EventLoop {
    pub async fn run<'js, T, R>(&self, ctx: &Ctx<'js>, runner: T) -> rquickjs::Result<R>
    where
        T: AsyncFnOnce(Context<'js>) -> rquickjs::Result<R>,
        R: FromJs<'js>,
    {
        let runtime = Runtime::from_ctx(ctx)?;
        let executor = TaskExecutor::new(&*runtime.borrow());
        executor.run_async(ctx, ResourceKind::ROOT, runner).await
    }
}

pub trait AsyncRunner<'js> {
    type Output: FromJs<'js>;
    fn run(self, ctx: Context<'js>) -> impl Future<Output = rquickjs::Result<Self::Output>>;
}
