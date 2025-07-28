use futures::{FutureExt, pin_mut};
use klaver_util::{
    RuntimeError,
    rquickjs::{self, AsyncContext, CatchResultExt, Ctx, FromJs},
};

use crate::{async_state::AsyncState, resource::TaskCtx};

pub struct EventLoop<T> {
    runner: T,
}

impl<T> EventLoop<T> {
    pub fn new(runner: T) -> EventLoop<T> {
        EventLoop { runner }
    }

    pub async fn run<R>(self, context: &AsyncContext) -> Result<R, RuntimeError>
    where
        T: for<'js> Runner<'js, Output = R>,
        R: 'static,
    {
        let state = context
            .with(|ctx| Result::<_, RuntimeError>::Ok(AsyncState::get(&ctx).catch(&ctx)?.clone()))
            .await?;

        let work = rquickjs::async_with!(context => |ctx| {
            let state = AsyncState::get(&ctx).catch(&ctx)?;

            let ret = state.run(ctx.clone(), |ctx| {
                self.runner.run(ctx)
            }).await.catch(&ctx)?;

            Result::<_, RuntimeError>::Ok(ret)
        });

        work.await
    }
}

pub trait Runner<'js> {
    type Output: FromJs<'js>;
    fn run(&self, ctx: TaskCtx<'js>) -> impl Future<Output = rquickjs::Result<Self::Output>>;
}
