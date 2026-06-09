use klaver_util::{Core, Map, ObjectExt, RuntimeError};
use rquickjs::{AsyncContext, AsyncRuntime, CatchResultExt, Function, Object};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), RuntimeError> {
    let runtime = AsyncRuntime::new()?;
    let context = AsyncContext::full(&runtime).await?;

    rquickjs::async_with!(context => |ctx| {

        let base: Object = ctx.eval("new Map")?;

        let core = Core::instance(&ctx)?;



        let test: Function = ctx.eval("(map) => { map.set('Hello', 42) }").catch(&ctx)?;

        test.call::<_, ()>((base.clone(),))?;

        println!("{}", base.call_property::<_,_, i32>("get", ("Hello",))?);


      Result::<_, RuntimeError>::Ok(())
    })
    .await?;

    runtime.idle().await;

    Ok(())
}
