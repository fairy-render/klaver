use klaver_modules::{transformer::SwcTranspiler, Builder, GlobalInfo};
use rquickjs::{prelude::Func, AsyncContext, AsyncRuntime, CatchResultExt, Ctx, Module};

struct TestGlobal;

impl GlobalInfo for TestGlobal {
    fn register(builder: &mut klaver_modules::GlobalBuilder<'_, Self>) {
        builder.register(|ctx: Ctx<'_>| {
            //
            ctx.globals().set("TEST_GLOBAL", "Hello, from global!")?;
            Ok(())
        });
    }
}

fn main() -> rquickjs::Result<()> {
    futures::executor::block_on(async move {
        let builder = Builder::default();

        let env = builder
            .search_path(std::env::current_dir().unwrap())
            .global::<TestGlobal>()
            .transpiler(SwcTranspiler::new())
            .build();

        let runtime = env.create_runtime().await.unwrap();
        let context = AsyncContext::full(&runtime).await.unwrap();

        // env.init(&context).await.unwrap();

        rquickjs::async_with!(context => |ctx| {
            ctx.globals().set(
                "print",
                Func::new(|msg: String| {
                    println!("{msg}");
                    rquickjs::Result::Ok(())
                }),
            )?;

            Module::import(&ctx, "./rquickjs-modules/examples/test.js")?
                .into_future::<()>().await
                .catch(&ctx)
                .unwrap();

            rquickjs::Result::Ok(())
        })
        .await
        .unwrap();

        runtime.execute_pending_job().await
    })
    .unwrap();

    Ok(())
}
