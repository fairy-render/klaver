use futures::FutureExt;
use klaver_util::{
    RuntimeError,
    rquickjs::{self, AsyncContext, CatchResultExt, FromJs},
};

use crate::{AsyncState, Context};

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
        R: for<'js> FromJs<'js>,
    {
        let work = rquickjs::async_with!(context => |ctx| {

            let ret = AsyncState::run_async(&ctx, |ctx| async {
                self.runner.run(ctx).await
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
    fn run(self, ctx: Context<'js>) -> impl Future<Output = rquickjs::Result<Self::Output>>;
}
