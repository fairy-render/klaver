use klaver_runtime::{AsyncState, Context, Resource, ResourceId, set_promise_hook};
use klaver_util::{
    RuntimeError,
    rquickjs::{
        self, AsyncContext, AsyncRuntime, CatchResultExt, Ctx, Function, Module, Value,
        prelude::{Func, Opt, Rest},
    },
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), RuntimeError> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::level_filters::LevelFilter::TRACE)
        .init();
    let runtime = AsyncRuntime::new()?;
    let context = AsyncContext::full(&runtime).await?;

    set_promise_hook(&runtime).await;

    rquickjs::async_with!(context => |ctx| {

      run(ctx.clone()).await.catch(&ctx)?;
      Result::<_, RuntimeError>::Ok(())
    })
    .await?;

    Ok(())
}
async fn run<'js>(ctx: Ctx<'js>) -> rquickjs::Result<()> {
    AsyncState::run(&ctx, |ctx: Context<'js>| async move {
        run_inner(ctx.ctx().clone()).await
    })
    .await
}

async fn run_inner<'js>(ctx: Ctx<'js>) -> rquickjs::Result<()> {
    ctx.globals().set(
        "print",
        Func::new(|ctx: Ctx<'js>, value: Rest<Value<'js>>| {
            let mut output = String::new();
            for (k, v) in value.0.iter().enumerate() {
                if k > 0 {
                    output.push_str(" ");
                }

                klaver_util::format_to(&ctx, v, &mut output, Default::default())?;
            }

            println!("{output}");

            rquickjs::Result::Ok(())
        }),
    )?;

    ctx.globals().set(
        "gc",
        Func::new(|ctx: Ctx<'js>| {
            ctx.run_gc();
        }),
    )?;

    ctx.globals().set(
        "testAsync",
        Func::new(|ctx: Ctx<'js>, cb: Function<'js>| {
            //

            AsyncState::push(&ctx, TestResource { callback: cb })?;

            rquickjs::Result::Ok(())
        }),
    )?;

    ctx.globals().set(
        "setTimeout",
        Func::new(|ctx: Ctx<'js>, cb: Function<'js>, timeout: Opt<u64>| {
            //

            AsyncState::push(
                &ctx,
                TimeResource {
                    callback: cb,
                    timeout: timeout.unwrap_or_default(),
                },
            )?;

            rquickjs::Result::Ok(())
        }),
    )?;

    Module::declare_def::<klaver_runtime::TaskModule, _>(ctx.clone(), "node:async_hooks")?;

    let module = Module::declare(ctx.clone(), "main", include_str!("./store.js"))?;

    module.meta()?.set("main", true)?;

    let (module, promise) = module.eval()?;

    // let ret = Module::evaluate(ctx.ctx().clone(), "main", include_str!("./test.js"))?
    //     .into_future::<()>()
    //     .await;

    promise.into_future().await
}

// pub struct TestRunner;

// impl<'js> Runner<'js> for TestRunner {
//     type Output = ();
//     async fn run(
//         self,
//         ctx: klaver_runtime::TaskCtx<'js>,
//     ) -> klaver_util::rquickjs::Result<Self::Output> {
//         ctx.globals().set(
//             "print",
//             Func::new(|ctx: Ctx<'js>, value: Rest<Value<'js>>| {
//                 let mut output = String::new();
//                 for (k, v) in value.0.iter().enumerate() {
//                     if k > 0 {
//                         output.push_str(" ");
//                     }

//                     klaver_util::format_to(&ctx, v, &mut output, Default::default())?;
//                 }

//                 println!("{output}");

//                 rquickjs::Result::Ok(())
//             }),
//         )?;

//         ctx.globals().set(
//             "gc",
//             Func::new(|ctx: Ctx<'js>| {
//                 ctx.run_gc();
//             }),
//         )?;

//         ctx.globals().set(
//             "testAsync",
//             Func::new(|ctx: Ctx<'js>, cb: Function<'js>| {
//                 //

//                 AsyncState::push(&ctx, TestResource { callback: cb })?;

//                 rquickjs::Result::Ok(())
//             }),
//         )?;

//         ctx.globals().set(
//             "setTimeout",
//             Func::new(|ctx: Ctx<'js>, cb: Function<'js>, timeout: Opt<u64>| {
//                 //

//                 AsyncState::push(
//                     &ctx,
//                     TimeResource {
//                         callback: cb,
//                         timeout: timeout.unwrap_or_default(),
//                     },
//                 )?;

//                 rquickjs::Result::Ok(())
//             }),
//         )?;

//         Module::declare_def::<klaver_runtime::TaskModule, _>(
//             ctx.ctx().clone(),
//             "node:async_hooks",
//         )?;

//         let module = Module::declare(ctx.ctx().clone(), "main", include_str!("./test.js"))?;

//         module.meta()?.set("main", true)?;

//         let (module, promise) = module.eval()?;

//         // let ret = Module::evaluate(ctx.ctx().clone(), "main", include_str!("./test.js"))?
//         //     .into_future::<()>()
//         //     .await;

//         promise.into_future().await
//     }
// }

struct TestResource<'js> {
    callback: Function<'js>,
}

pub struct ResourceKey;

impl ResourceId for ResourceKey {
    fn name() -> &'static str {
        "Test"
    }
}

impl<'js> Resource<'js> for TestResource<'js> {
    type Id = ResourceKey;
    const SCOPED: bool = true;
    fn run(self, ctx: klaver_runtime::Context<'js>) -> impl Future<Output = rquickjs::Result<()>> {
        async move {
            ctx.invoke_callback::<_, ()>(self.callback.clone(), ())?;
            // ctx.wait_shutdown().await?;

            Ok(())
        }
    }
}

struct TimeResource<'js> {
    callback: Function<'js>,
    timeout: u64,
}

pub struct TimeoutKey;

impl ResourceId for TimeoutKey {
    fn name() -> &'static str {
        "Timeout"
    }
}

impl<'js> Resource<'js> for TimeResource<'js> {
    type Id = TimeoutKey;
    const SCOPED: bool = false;
    const INTERNAL: bool = false;
    fn run(self, ctx: klaver_runtime::Context<'js>) -> impl Future<Output = rquickjs::Result<()>> {
        async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(self.timeout)).await;

            ctx.invoke_callback::<_, ()>(self.callback.clone(), ())?;

            Ok(())
        }
    }
}
