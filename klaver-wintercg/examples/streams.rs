use rquickjs::{AsyncContext, AsyncRuntime, CatchResultExt, Module};
use rquickjs_modules::Builder;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), rquickjs::Error> {
    let runtime = AsyncRuntime::new()?;
    let context = AsyncContext::full(&runtime).await?;

    let modules = Builder::new()
        .global::<klaver_wintercg::Globals>()
        .search_path(".")
        .build();

    modules.init(&context).await?;

    let source = include_str!("./stream.js");

    rquickjs::async_with!(context => |ctx| {
        Module::evaluate(ctx.clone(), "main.js", source)?
            .into_future::<()>()
            .await?;


        rquickjs::Result::Ok(())
    })
    .await?;

    Ok(())
}
