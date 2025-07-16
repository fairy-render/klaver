use klaver_base::streams::WritableStream;
use klaver_runner::{FuncFn, Runner};
use rquickjs::{
    AsyncContext, AsyncRuntime, CatchResultExt, Class, Module, class::JsClass, prelude::Func,
};
use rquickjs_util::{RuntimeError, StringRef};

fn main() -> Result<(), RuntimeError> {
    futures::executor::block_on(async move {
        let runtime = AsyncRuntime::new()?;

        let context = AsyncContext::full(&runtime).await?;

        Runner::new(
            &context,
            FuncFn::new(|ctx, worker| {
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

                    let (_, promise) = Module::evaluate_def::<klaver_base::BaseModule, _>(
                        ctx.clone(),
                        "quick:base",
                    )
                    .catch(&ctx)?;

                    promise.into_future::<()>().await.catch(&ctx)?;

                    let (_, promise) =
                        Module::declare(ctx.clone(), "main", include_str!("./test.js"))?
                            .eval()
                            .catch(&ctx)?;

                    promise.into_future::<()>().await.catch(&ctx)?;

                    Result::<_, RuntimeError>::Ok(())
                })
            }),
        )
        .run()
        .await?;

        // runtime.idle().await;

        Ok(())
    })
}
