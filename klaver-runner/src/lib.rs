mod shutdown;
mod uncaught_exeception;
mod workers;

use futures::{FutureExt, future::LocalBoxFuture, pin_mut};
use rquickjs::{AsyncContext, Ctx};
use rquickjs_util::RuntimeError;

pub use self::shutdown::Shutdown;
pub use crate::workers::Workers;

pub struct Runner<'a, T: ?Sized> {
    context: &'a AsyncContext,
    func: T,
}

impl<'a, T: 'a> Runner<'a, T> {
    pub fn new(context: &'a AsyncContext, func: T) -> Runner<'a, T> {
        Runner { context, func }
    }

    pub async fn run(self) -> Result<(), RuntimeError>
    where
        for<'b> T: Runnerable + 'b,
    {
        let workers = Workers::new();

        // Start worker function
        let worker_clone = workers.clone();
        let work_future = rquickjs::async_with!(self.context => |ctx| {
            ctx.store_userdata(worker_clone.clone()).map_err(|err| RuntimeError::Message(Some(err.to_string())))?;
            self.func.call(ctx, worker_clone).await
        }).fuse();
        pin_mut!(work_future);

        // Create waiting future
        let wait_future = workers.wait().fuse();
        pin_mut!(wait_future);

        // Create drive future
        let drive = self.context.runtime().drive().fuse();
        pin_mut!(drive);

        let mut work_done = false;

        let ret = loop {
            futures::select! {
                ret = work_future => {
                    work_done = true;
                    if let Err(err) = ret {
                        break Err(err)
                    }


                    // Send kill signal to workers
                    workers.shutdown();

                }
                _ = drive => {
                    if work_done {
                        break Ok(())
                    }
                }
                ret = wait_future => {
                    if work_done {
                        break ret.map_err(Into::into)
                    } else if let Err(err) = ret {
                        break Err(err.into())
                    }
                }
            };
        };

        self.context
            .with(|ctx| {
                ctx.remove_userdata::<Workers>()?;
                rquickjs::Result::Ok(())
            })
            .await?;

        ret
    }
}

pub trait Runnerable {
    type Future<'js>: Future<Output = Result<(), RuntimeError>>
    where
        Self: 'js;

    fn call<'js>(self, ctx: Ctx<'js>, worker: Workers) -> Self::Future<'js>
    where
        Self: 'js;
}

impl<T> Runnerable for T
where
    T: 'static,
    T: for<'js> FnOnce(Ctx<'js>, Workers) -> LocalBoxFuture<'js, Result<(), RuntimeError>>,
{
    type Future<'js>
        = LocalBoxFuture<'js, Result<(), RuntimeError>>
    where
        Self: 'js;

    fn call<'js>(self, ctx: Ctx<'js>, worker: Workers) -> Self::Future<'js>
    where
        Self: 'js,
    {
        (self)(ctx, worker)
    }
}

pub struct FuncFn<T>(pub T);

impl<T> FuncFn<T>
where
    T: for<'js> FnOnce(Ctx<'js>, Workers) -> LocalBoxFuture<'js, Result<(), RuntimeError>>,
{
    pub fn new(func: T) -> FuncFn<T> {
        FuncFn(func)
    }
}

impl<T> Runnerable for FuncFn<T>
where
    T: 'static,
    T: for<'js> FnOnce(Ctx<'js>, Workers) -> LocalBoxFuture<'js, Result<(), RuntimeError>>,
{
    type Future<'js>
        = LocalBoxFuture<'js, Result<(), RuntimeError>>
    where
        Self: 'js;

    fn call<'js>(self, ctx: Ctx<'js>, worker: Workers) -> Self::Future<'js>
    where
        Self: 'js,
    {
        (self.0)(ctx, worker)
    }
}
