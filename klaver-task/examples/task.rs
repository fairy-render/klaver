use klaver_task::{AsyncState, EventLoop, Hook, Resource, Runner, ScriptHook, Shutdown, get_hooks};
use klaver_util::{
    RuntimeError, format,
    rquickjs::{self, AsyncContext, AsyncRuntime, Ctx, Function, Module, Value, prelude::Func},
};

fn main() -> Result<(), RuntimeError> {
    futures::executor::block_on(async move {
        let runtime = AsyncRuntime::new()?;
        let context = AsyncContext::full(&runtime).await?;

        let event_loop = EventLoop::new(TestRunner);

        event_loop.run(&context).await?;

        Result::<_, RuntimeError>::Ok(())
    })?;

    Ok(())
}

struct TestRunner;

impl<'js> Runner<'js> for TestRunner {
    type Output = ();

    async fn run(&self, ctx: rquickjs::Ctx<'js>) -> rquickjs::Result<Self::Output> {
        ctx.globals().set(
            "print",
            Func::new(|ctx: Ctx<'js>, value: Value<'js>| {
                let str = format(&ctx, &value, Default::default())?;

                println!("{str}");

                rquickjs::Result::Ok(())
            }),
        )?;

        ctx.globals().set(
            "currentAsyncId",
            Func::new(|ctx: Ctx<'js>| {
                let tasks = AsyncState::get(&ctx)?;

                let id = tasks.current_async_id();

                rquickjs::Result::Ok(id)
            }),
        )?;

        ctx.globals().set(
            "currentExecutionId",
            Func::new(|ctx: Ctx<'js>| {
                let tasks = AsyncState::get(&ctx)?;

                let id = tasks.execution_id();

                rquickjs::Result::Ok(id)
            }),
        )?;

        ctx.globals().set(
            "testAsync",
            Func::new(|ctx: Ctx<'js>, cb: Function<'js>| {
                //

                let tasks = AsyncState::get(&ctx)?;

                tasks.push(ctx.clone(), TestResource { callback: cb })?;

                rquickjs::Result::Ok(())
            }),
        )?;

        ctx.globals().set(
            "createHook",
            Func::new(|ctx: Ctx<'js>, hook: ScriptHook<'js>| {
                let hooks = get_hooks(&ctx)?;

                hooks.borrow_mut().add_listener(Hook::Script(hook));

                rquickjs::Result::Ok(())
            }),
        )?;

        Module::evaluate(ctx, "main", include_str!("./test.js"))?
            .into_future::<()>()
            .await?;

        Ok(())
    }
}

struct TestResource<'js> {
    callback: Function<'js>,
}

impl<'js> Resource<'js> for TestResource<'js> {
    fn run(
        &self,
        ctx: klaver_task::TaskCtx<'js>,
        shutdown: Shutdown,
    ) -> impl Future<Output = rquickjs::Result<()>> {
        async move {
            println!("Running!");
            ctx.invoke_callback::<_, ()>(self.callback.clone(), ())?;

            Ok(())
        }
    }
}
