use rquickjs::{
    prelude::Func, AsyncContext, AsyncRuntime, CatchResultExt, Ctx, Module,
};
use rquickjs_modules::{
    transformer::Compiler,
    Builder, GlobalInfo,
};

struct TestGlobal;

impl GlobalInfo for TestGlobal {
    fn register(builder: &mut rquickjs_modules::GlobalBuilder<'_, Self>) {
        builder.register(|ctx: Ctx<'_>| {
            //
            ctx.globals().set("TEST_GLOBAL", "Hello, from global!")?;
            Ok(())
        });
    }
}

fn main() -> rquickjs::Result<()> {
    futures::executor::block_on(async move {
        let runtime = AsyncRuntime::new().unwrap();
        let context = AsyncContext::full(&runtime).await.unwrap();

        let compiler = Compiler::default();
        // compiler.transform_options.

        let builder = Builder::default();

        let env = builder
            .search_path(std::env::current_dir().unwrap())
            .global::<TestGlobal>()
            .compiler(compiler)
            .build();

        env.init(&context).await.unwrap();

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
