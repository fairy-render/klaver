use futures::{FutureExt, pin_mut};
use klaver_util::{
    RuntimeError,
    rquickjs::{self, AsyncContext, CatchResultExt, Ctx, FromJs},
};

use crate::state::AsyncState;

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
            let ret = self.runner.run(ctx.clone()).await.catch(&ctx)?;
            Result::<_, RuntimeError>::Ok(ret)
        });
        pin_mut!(work);
        let mut work = work.fuse();

        let wait = state.wait();
        pin_mut!(wait);
        let mut wait = wait.fuse();

        let mut resp: Option<R> = None;

        let mut wait_done = false;

        let ret = loop {
            futures::select! {
                ret = &mut work => {
                    match ret {
                        Ok(ret) => {
                            if wait_done {
                                break Ok(ret)
                            }
                            resp = Some(ret)
                        }
                        Err(err) => {
                            break Err(err)
                        }
                    }
                },
                ret = &mut wait => {
                    wait_done = true;
                    match ret {
                        Ok(()) => {
                            if let Some(resp) = resp.take() {
                                break Ok(resp)
                            }
                        }
                        Err(err) => {
                            break Err(err.into())
                        }
                    }
                }

            };
        };

        ret
    }
}

pub trait Runner<'js> {
    type Output: FromJs<'js>;
    fn run(&self, ctx: Ctx<'js>) -> impl Future<Output = rquickjs::Result<Self::Output>>;
}
