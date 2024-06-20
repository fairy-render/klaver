use klaver_base::get_config;
use klaver_module::{Modules};
use rquickjs::{
    AsyncContext, AsyncRuntime, Error, Function, Module,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let runtime = AsyncRuntime::new()?;
    let context = AsyncContext::full(&runtime).await?;

    let mut modules = Modules::default();

    modules.register::<klaver_os::env::Moodule>("@klaver/env");
    modules.register::<klaver_os::shell::Module>("@klaver/shell");
    modules.register::<klaver_http::Module>("@klaver/http");
    modules.register::<klaver_base::Module>("@klaver/base");

    modules.add_search_path(std::env::current_dir().unwrap().display().to_string());

    modules.attach(&runtime).await;

    let args = std::env::args().skip(1).collect::<Vec<_>>();

    if args.is_empty() {
        eprintln!("Usage: {} <file>", std::env::args().next().unwrap());
        return Ok(());
    }

    let content = std::fs::read_to_string(&args[0])?;

    let (content, _) = klaver_module::typescript::compile(&args[0], &content);

    let ret = context
        .with(|ctx| {
            ctx.globals().set(
                "print",
                Function::new(ctx.clone(), |arg: rquickjs::Value| {
                    println!("{}", arg.try_into_string().unwrap().to_string()?);
                    rquickjs::Result::Ok(())
                }),
            )?;

            klaver_compat::init(&ctx)?;

            get_config(&ctx, |config| {
                config.set_cwd(|| std::env::current_dir().ok());
                config.set_args(|| std::env::args().collect());
                Ok(())
            })?;

            let globals = ctx.globals();

            let module = Module::evaluate(ctx.clone(), "main", &*content)?;

            let _ = module.finish::<()>()?;

            rquickjs::Result::Ok(())
        })
        .await;

    runtime.idle().await;

    if let Err(Error::Exception) = ret {
        context
            .with(|ctx| {
                let catch = ctx.catch();

                if !catch.is_null() {
                    println!(
                        "catch: {:?}",
                        catch.try_into_exception().unwrap().to_string()
                    );
                }

                rquickjs::Result::Ok(())
            })
            .await?;
    }

    Ok(())
}
