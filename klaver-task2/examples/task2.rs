use klaver_task2::{AsyncState, EventLoop, Resource, Runner};
use klaver_util::{
    RuntimeError,
    rquickjs::{
        self, AsyncContext, AsyncRuntime, Ctx, Function, Module, Value,
        prelude::{Func, Rest},
    },
};

fn main() -> Result<(), RuntimeError> {
    futures::executor::block_on(async move {
        let runtime = AsyncRuntime::new()?;
        let context = AsyncContext::full(&runtime).await?;

        EventLoop::new(TestRunner).run(&context).await?;

        Ok(())
    })
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
            "testAsync",
            Func::new(|ctx: Ctx<'js>, cb: Function<'js>| {
                //

                let tasks = AsyncState::get(&ctx)?;

                tasks.push(&ctx, TestResource { callback: cb })?;

                rquickjs::Result::Ok(())
            }),
        )?;

        Module::declare_def::<klaver_task2::TaskModule, _>(ctx.ctx.clone(), "node:async_hooks")?;

        Module::evaluate(ctx.ctx.clone(), "main", include_str!("./test.js"))?
            .into_future::<()>()
            .await?;

        println!("Run");
        Ok(())
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
            println!("Running!");
            ctx.invoke_callback::<_, ()>(self.callback.clone(), ())?;

            Ok(())
        }
    }
}
