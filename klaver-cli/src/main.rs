use klaver::{
    quick::{CatchResultExt, Module},
    vm::VmOptions,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    /*let runtime = AsyncRuntime::new()?;
    let context = AsyncContext::full(&runtime).await?;

    let mut modules = Modules::default();

    modules.register::<klaver_os::env::Moodule>("@klaver/env");
    modules.register::<klaver_os::shell::Module>("@klaver/shell");
    modules.register::<klaver_http::Module>("@klaver/http");
    modules.register::<klaver_base::Module>("@klaver/base");

    modules.add_search_path(std::env::current_dir().unwrap().display().to_string());

    modules.attach(&runtime).await;*/

    let vm = VmOptions::default()
        .search_path(".")
        .module::<klaver_encoding::Encoding>()
        .module::<klaver_compat::Compat>()
        .build()
        .await?;

    klaver_compat::init(&vm).await?;

    let args = std::env::args().skip(1).collect::<Vec<_>>();

    if args.is_empty() {
        eprintln!("Usage: {} <file>", std::env::args().next().unwrap());
        return Ok(());
    }

    let content = std::fs::read_to_string(&args[0])?;

    // let (content, _) = klaver_module::typescript::compile(&args[0], &content);

    let ret = klaver::async_with!(vm => |ctx| {

        let _ = Module::evaluate(ctx.clone(), "main", content).catch(&ctx)?.into_future::<()>().await.catch(&ctx)?;

       Ok(())
    })
    .await?;

    vm.idle().await?;

    // if let Err(Error::Exception) = ret {
    //     context
    //         .with(|ctx| {
    //             let catch = ctx.catch();

    //             if !catch.is_null() {
    //                 println!(
    //                     "catch: {:?}",
    //                     catch.try_into_exception().unwrap().to_string()
    //                 );
    //             }

    //             rquickjs::Result::Ok(())
    //         })
    //         .await?;
    // }

    // runtime.idle().await;

    // let ret = context
    //     .with(|ctx| {
    //         let base = get_base(&ctx)?;
    //         let mut base = base.try_borrow_mut()?;

    //         base.uncaught(ctx)
    //     })
    //     .await;

    // if let Err(Error::Exception) = ret {
    //     context
    //         .with(|ctx| {
    //             let catch = ctx.catch();

    //             if !catch.is_null() {
    //                 println!(
    //                     "catch: {:?}",
    //                     catch
    //                         .try_into_exception()
    //                         .map(|m| m.to_string())
    //                         .or_else(|v| v
    //                             .try_into_string()
    //                             .map_err(|_| rquickjs::Error::new_from_js("not", "to"))
    //                             .and_then(|m| m.to_string()))
    //                         .unwrap()
    //                 );
    //             }

    //             rquickjs::Result::Ok(())
    //         })
    //         .await?;
    // }

    Ok(())
}
