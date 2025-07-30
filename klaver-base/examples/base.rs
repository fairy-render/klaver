use klaver_base::{AbortSignal, Emitter, EventTarget, streams::WritableStream};
use klaver_task::{EventLoop, Runner};
use klaver_util::{Inheritable, RuntimeError, StringRef, Subclass, SuperClass, is_plain_object};
use rquickjs::{
    AsyncContext, AsyncRuntime, CatchResultExt, Class, Ctx, Module, Value, class::JsClass,
    prelude::Func,
};

fn main() -> Result<(), RuntimeError> {
    futures::executor::block_on(async move {
        let runtime = AsyncRuntime::new()?;

        let context = AsyncContext::full(&runtime).await?;

        EventLoop::new(Base).run(&context).await?;

        Ok(())
    })
}

struct Base;

impl<'js> Runner<'js> for Base {
    type Output = ();
    async fn run(self, ctx: klaver_task::TaskCtx<'js>) -> rquickjs::Result<Self::Output> {
        AbortSignal::inherit(&ctx)?;

        let signal = Class::instance(ctx.ctx().clone(), AbortSignal::new()?)?
            .into_value()
            .into_object()
            .unwrap();

        ctx.globals().set(
            "print",
            Func::from(|msg: StringRef<'_>| {
                println!("{}", msg);
                rquickjs::Result::Ok(())
            }),
        )?;

        let (_, promise) =
            Module::evaluate_def::<klaver_base::BaseModule, _>(ctx.ctx().clone(), "quick:base")?;

        promise.into_future::<()>().await?;

        let (_, promise) =
            Module::declare(ctx.ctx().clone(), "main", include_str!("./test.js"))?.eval()?;

        promise.into_future::<()>().await?;

        Ok(())
    }
}
