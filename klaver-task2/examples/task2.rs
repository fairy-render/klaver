use klaver_task2::{AsyncState, EventLoop, Resource, Runner, set_promise_hook};
use klaver_util::{
    RuntimeError,
    rquickjs::{
        self, AsyncContext, AsyncRuntime, Ctx, Function, Module, Value,
        prelude::{Func, Opt, Rest},
    },
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), RuntimeError> {
    let runtime = AsyncRuntime::new()?;
    let context = AsyncContext::full(&runtime).await?;

    set_promise_hook(&runtime).await;

    EventLoop::new(TestRunner).run(&context).await?;

    Ok(())
}

pub struct TestRunner;

impl<'js> Runner<'js> for TestRunner {
    type Output = ();
    async fn run(
        &self,
        ctx: klaver_task2::TaskCtx<'js>,
    ) -> klaver_util::rquickjs::Result<Self::Output> {
        ctx.ctx.globals().set(
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

        ctx.ctx.globals().set(
            "gc",
            Func::new(|ctx: Ctx<'js>| {
                ctx.run_gc();
            }),
        )?;

        ctx.ctx.globals().set(
            "testAsync",
            Func::new(|ctx: Ctx<'js>, cb: Function<'js>| {
                //

                let tasks = AsyncState::get(&ctx)?;

                tasks.push(&ctx, TestResource { callback: cb })?;

                rquickjs::Result::Ok(())
            }),
        )?;

        ctx.ctx.globals().set(
            "setTimeout",
            Func::new(|ctx: Ctx<'js>, cb: Function<'js>, timeout: Opt<u64>| {
                //

                let tasks = AsyncState::get(&ctx)?;

                tasks.push(
                    &ctx,
                    TimeResource {
                        callback: cb,
                        timeout: timeout.unwrap_or_default(),
                    },
                )?;

                rquickjs::Result::Ok(())
            }),
        )?;

        Module::declare_def::<klaver_task2::TaskModule, _>(ctx.ctx.clone(), "node:async_hooks")?;

        let ret = Module::evaluate(ctx.ctx.clone(), "main", include_str!("./test.js"))?
            .into_future::<()>()
            .await;

        println!("Run");
        ret
    }
}

struct TestResource<'js> {
    callback: Function<'js>,
}

impl<'js> Resource<'js> for TestResource<'js> {
    fn ty(&self) -> &str {
        "Test"
    }
    fn run(&self, ctx: klaver_task2::TaskCtx<'js>) -> impl Future<Output = rquickjs::Result<()>> {
        async move {
            ctx.invoke_callback::<_, ()>(self.callback.clone(), ())?;
            ctx.wait_shutdown().await?;

            Ok(())
        }
    }
}

struct TimeResource<'js> {
    callback: Function<'js>,
    timeout: u64,
}

impl<'js> Resource<'js> for TimeResource<'js> {
    fn ty(&self) -> &str {
        "Timeout"
    }
    fn run(&self, ctx: klaver_task2::TaskCtx<'js>) -> impl Future<Output = rquickjs::Result<()>> {
        async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(self.timeout)).await;

            ctx.invoke_callback::<_, ()>(self.callback.clone(), ())?;

            Ok(())
        }
    }
}
