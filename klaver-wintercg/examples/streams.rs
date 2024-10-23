use klaver::{Vm, VmOptions};
use rquickjs::{AsyncContext, AsyncRuntime, CatchResultExt, Module};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), klaver::Error> {
    let vm = VmOptions::default()
        .module::<klaver_wintercg::Module>()
        .build()
        .await?;

    let source = include_str!("./stream.js");

    klaver_wintercg::install_globals(&vm).await?;

    klaver::async_with!(vm => |ctx| {
        Module::evaluate(ctx.clone(), "main.js", source)
            .catch(&ctx)?
            .into_future::<()>()
            .await
            .catch(&ctx)?;
        Ok(())
    })
    .await?;

    Ok(())
}
