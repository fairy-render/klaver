use klaver_wintercg::wait_timers;
use rquickjs::{
    class::Trace, AsyncContext, AsyncRuntime, CatchResultExt, IntoJs, Module, Object, Value,
};
use rquickjs_modules::Builder;
use rquickjs_util::{create_proxy, ProxyHandler};

#[derive(Trace)]
struct TestProxy;

impl<'js> ProxyHandler<'js, Object<'js>> for TestProxy {
    fn get(
        &self,
        ctx: rquickjs::Ctx<'js>,
        target: Object<'js>,
        prop: rquickjs_util::Prop<'js>,
        receiver: rquickjs::Value<'js>,
    ) -> rquickjs::Result<rquickjs::Value<'js>> {
        let prop = prop.to_string(&ctx)?;
        if prop.as_str() == "ost" {
            rquickjs::String::from_str(ctx.clone(), "Hello, World!")?.into_js(&ctx)
        } else {
            Ok(Value::new_undefined(ctx))
        }
    }
}

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

        let proxy = create_proxy(ctx.clone(), Object::new(ctx.clone())?, TestProxy).catch(&ctx)?;

        ctx.globals().set("dims", proxy).catch(&ctx)?;

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
