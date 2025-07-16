use futures::future::LocalBoxFuture;
use klaver_runner::{Func, FuncFn, Runner, Workers};
use rquickjs::{AsyncContext, AsyncRuntime, CatchResultExt, Ctx};
use rquickjs_util::RuntimeError;

pub struct Test<'a>(&'a str);

impl<'a> Func for Test<'a> {
    type Future<'js> = LocalBoxFuture<'js, Result<(), RuntimeError>>;

    fn call<'js>(self, ctx: rquickjs::Ctx<'js>, workers: Workers) -> Self::Future<'js> {
        Box::pin(async move {
            //
            workers.push(ctx, |ctx, listener| async move {
                //

                println!("Started");
                ctx.eval::<(), _>("throw new Error('Js error')")
                    .catch(&ctx)?;
                listener.await;
                println!("finished");

                // Ok(())
                Ok(())
            });

            Ok(())
        })
    }
}

fn main() -> Result<(), RuntimeError> {
    futures::executor::block_on(async move {
        let runtime = AsyncRuntime::new()?;
        let context = AsyncContext::full(&runtime).await?;

        Runner::new(
            &context,
            FuncFn::new(|ctx, worker| {
                Box::pin(async move {
                    //
                    ctx.globals().set("Hello", "Hello")?;
                    Result::<_, RuntimeError>::Ok(())
                })
            }),
        )
        .run()
        .await?;

        Runner::new(&context, Test("Hello")).run().await?;

        Ok(())
    })
}
