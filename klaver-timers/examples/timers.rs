use futures::future::LocalBoxFuture;
use klaver_runtime::{AsyncState, Context, Execution, ExitMode};
use klaver_timers::{TimeId, Timers, TimingBackend};
use klaver_util::{FunctionExt, RuntimeError, StringRef};
use rquickjs::{
    AsyncContext, AsyncRuntime, CatchResultExt, Class, Ctx, Function, IntoJs, Module,
    prelude::{Func, Opt},
};

// pub struct Test;

// impl WorkerFunc for Test {
//     type Future<'js> = LocalBoxFuture<'js, Result<(), RuntimeError>>;

//     fn call<'js>(self, ctx: rquickjs::Ctx<'js>, _workers: Workers) -> Self::Future<'js> {
//         Box::pin(async move {
//             ctx.globals()
//                 .set(
//                     "print",
//                     Func::from(|msg: StringRef<'_>| {
//                         println!("{}", msg);
//                         rquickjs::Result::Ok(())
//                     }),
//                 )
//                 ?;

//             let timers = klaver_timers::Timers::new(ctx.clone())?;

//             let set_timeout = Func::new(
//                 |ctx: Ctx<'js>,
//                  timers: Class<'js, Timers<'js>>,
//                  func: Function<'js>,
//                  timeout: Opt<u64>| {
//                     timers
//                         .borrow_mut()
//                         .create_timeout(ctx, func, timeout, Opt(None))
//                 },
//             )
//             .into_js(&ctx)?
//             .get::<Function>()?
//             .bind(&ctx, (ctx.globals(), timers.clone()))?;

//             ctx.globals().set("setTimeout", set_timeout)?;

//             let clear_timeout = Func::new(|timers: Class<'js, Timers<'js>>, id: TimeId| {
//                 timers.borrow_mut().clear_timeout(id)
//             })
//             .into_js(&ctx)?
//             .get::<Function>()?
//             .bind(&ctx, (ctx.globals(), timers.clone()))?;

//             ctx.globals().set("clearTimeout", clear_timeout)?;

//             ctx.store_userdata(TimingBackend::new(TokioTimers).with_should_shutdown(false))
//                 .map_err(|err| RuntimeError::Custom(Box::from(err.to_string())))?;

//             let (_, promise) =
//                 Module::evaluate_def::<klaver_timers::TimeModule, _>(ctx.clone(), "timer")
//                     ?;
//             promise.into_future::<()>().await?;

//             let promise =
//                 Module::evaluate(ctx.clone(), "main", include_str!("./test.js"))?;

//             promise.into_future::<()>().await?;

//             Ok(())
//         })
//     }
// }

struct TokioTimers;

impl klaver_timers::Backend for TokioTimers {
    type Timer = tokio::time::Sleep;

    fn create_timer(&self, instant: std::time::Instant) -> Self::Timer {
        tokio::time::sleep_until(instant.into())
    }
}

async fn run<'js>(ctx: Ctx<'js>) -> rquickjs::Result<()> {
    ctx.globals().set(
        "print",
        Func::from(|msg: StringRef<'_>| {
            println!("{}", msg);
            rquickjs::Result::Ok(())
        }),
    )?;

    let timers = klaver_timers::Timers::new(ctx.clone())?;

    let set_timeout = Func::new(
        |ctx: Ctx<'js>, timers: Class<'js, Timers>, func: Function<'js>, timeout: Opt<u64>| {
            timers
                .borrow_mut()
                .create_timeout(ctx, func, timeout, Opt(None))
        },
    )
    .into_js(&ctx)?
    .get::<Function>()?
    .bind(&ctx, (ctx.globals(), timers.clone()))?;

    ctx.globals().set("setTimeout", set_timeout)?;

    let clear_timeout =
        Func::new(|timers: Class<'js, Timers>, id: TimeId| timers.borrow_mut().clear_timeout(id))
            .into_js(&ctx)?
            .get::<Function>()?
            .bind(&ctx, (ctx.globals(), timers.clone()))?;

    ctx.globals().set("clearTimeout", clear_timeout)?;

    ctx.store_userdata(TimingBackend::new(TokioTimers).with_should_shutdown(false))
        .unwrap();
    let (_, promise) = Module::evaluate_def::<klaver_timers::TimeModule, _>(ctx.clone(), "timer")?;
    promise.into_future::<()>().await?;

    let promise = Module::evaluate(ctx.clone(), "main", include_str!("./test.js"))?;

    promise.into_future::<()>().await?;
    Ok(())
}

async fn _run<'js>(ctx: Context<'js>) -> rquickjs::Result<()> {
    run(ctx.ctx().clone()).await
}

#[tokio::main]
async fn main() -> Result<(), RuntimeError> {
    let runtime = AsyncRuntime::new()?;
    let context = AsyncContext::full(&runtime).await?;

    rquickjs::async_with!(context => |ctx| {

        AsyncState::run_async_with(&ctx, Execution::default().wait(true).exit(ExitMode::Idle), |ctx| async {
            _run(ctx).await
        }).await.catch(&ctx)?;
        Result::<_, RuntimeError>::Ok(())
    })
    .await?;

    runtime.idle().await;

    Ok(())
}
