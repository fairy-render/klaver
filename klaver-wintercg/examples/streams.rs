use klaver_wintercg::wait_timers;
use rquickjs::{AsyncContext, AsyncRuntime, CatchResultExt, Module};
use rquickjs_modules::Builder;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), klaver_wintercg::RuntimeError> {
    let runtime = AsyncRuntime::new()?;
    let context = AsyncContext::full(&runtime).await?;

    let modules = Builder::new()
        .global::<klaver_wintercg::Globals>()
        .search_path(".")
        .build();

    modules.init(&context).await?;

    let source = include_str!("./stream.js");

    klaver_wintercg::run!(context => |ctx| {

        Module::evaluate(ctx.clone(), "main.js", source)?
            .into_future::<()>()
            .await.catch(&ctx)?;


        Ok(())
    })
    .await?;
    let now = std::time::Instant::now();
    wait_timers(&context).await?;
    println!("Since: {:?}", now.elapsed());

    Ok(())
}
