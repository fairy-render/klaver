use klaver_base::{
    AbortSignal, Emitter, EventTarget, Exportable, Registry, streams::WritableStream,
};
use klaver_runtime::{EventLoop, Runner};
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
    async fn run(self, ctx: klaver_runtime::Context<'js>) -> rquickjs::Result<Self::Output> {
        AbortSignal::inherit(&ctx)?;

        ctx.globals().set(
            "print",
            Func::from(|msg: StringRef<'_>| {
                println!("{}", msg);
                rquickjs::Result::Ok(())
            }),
        )?;

        klaver_base::BaseModule::export(&ctx, &Registry::instance(&ctx)?, &ctx.globals())?;

        let (_, promise) =
            Module::declare(ctx.ctx().clone(), "main", include_str!("./test.js"))?.eval()?;

        promise.into_future::<()>().await?;

        Ok(())
    }
}
