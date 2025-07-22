use futures::future::LocalBoxFuture;
use klaver_runner::{Runner, Runnerable as WorkerFunc, Workers};
use klaver_timers::{TimeId, Timers, TimingBackend};
use rquickjs::{
    AsyncContext, AsyncRuntime, CatchResultExt, Class, Ctx, Function, IntoJs, Module,
    prelude::{Func, Opt},
};
use rquickjs_util::{RuntimeError, StringRef, util::FunctionExt};

pub struct Test;

impl WorkerFunc for Test {
    type Future<'js> = LocalBoxFuture<'js, Result<(), RuntimeError>>;

    fn call<'js>(self, ctx: rquickjs::Ctx<'js>, _workers: Workers) -> Self::Future<'js>
    where
        Self: 'js,
    {
        Box::pin(async move {
            ctx.globals()
                .set(
                    "print",
                    Func::from(|msg: StringRef<'_>| {
                        println!("{}", msg);
                        rquickjs::Result::Ok(())
                    }),
                )
                .catch(&ctx)?;

            let timers = klaver_timers::Timers::new(ctx.clone())?;

            let set_timeout = Func::new(
                |ctx: Ctx<'js>,
                 timers: Class<'js, Timers<'js>>,
                 func: Function<'js>,
                 timeout: Opt<u64>| {
                    timers
                        .borrow_mut()
                        .create_timeout(ctx, func, timeout, Opt(None))
                },
            )
            .into_js(&ctx)?
            .get::<Function>()?
            .bind(ctx.clone(), (ctx.globals(), timers.clone()))?;

            ctx.globals().set("setTimeout", set_timeout)?;

            let clear_timeout = Func::new(|timers: Class<'js, Timers<'js>>, id: TimeId| {
                timers.borrow_mut().clear_timeout(id)
            })
            .into_js(&ctx)?
            .get::<Function>()?
            .bind(ctx.clone(), (ctx.globals(), timers.clone()))?;

            ctx.globals().set("clearTimeout", clear_timeout)?;

            ctx.store_userdata(TimingBackend::new(TokioTimers).with_should_shutdown(false))
                .map_err(|err| RuntimeError::Custom(Box::from(err.to_string())))?;

            let (_, promise) =
                Module::evaluate_def::<klaver_timers::TimeModule, _>(ctx.clone(), "timer")
                    .catch(&ctx)?;
            promise.into_future::<()>().await.catch(&ctx)?;

            let promise =
                Module::evaluate(ctx.clone(), "main", include_str!("./test.js")).catch(&ctx)?;

            promise.into_future::<()>().await.catch(&ctx)?;

            Ok(())
        })
    }
}

struct TokioTimers;

impl klaver_timers::Backend for TokioTimers {
    type Timer = tokio::time::Sleep;

    fn create_timer(&self, instant: std::time::Instant) -> Self::Timer {
        tokio::time::sleep_until(instant.into())
    }
}

#[tokio::main]
async fn main() -> Result<(), RuntimeError> {
    let runtime = AsyncRuntime::new()?;
    let context = AsyncContext::full(&runtime).await?;

    Runner::new(&context, Test).run().await
}
