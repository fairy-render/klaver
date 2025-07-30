use futures::FutureExt;
use klaver_util::{
    RuntimeError,
    rquickjs::{self, AsyncContext, CatchResultExt, FromJs},
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
        let work = rquickjs::async_with!(context => |ctx| {
            let state = AsyncState::instance(&ctx).catch(&ctx)?;

            let ret = state.run(ctx.clone(), |ctx| {
                self.runner.run(ctx)
            }).await.catch(&ctx)?;




            Result::<_, RuntimeError>::Ok(ret)
        });

        futures::select! {
            ret = work.fuse() => {
                ret
            }
            _ = context.runtime().drive().fuse() => {
                panic!()
            }
        }
    }
}

pub trait Runner<'js> {
    type Output: FromJs<'js>;
    fn run(self, ctx: TaskCtx<'js>) -> impl Future<Output = rquickjs::Result<Self::Output>>;
}
