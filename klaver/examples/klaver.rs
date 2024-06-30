use klaver::{vm::VmOptions, Error};
use rquickjs::CatchResultExt;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    let vm = VmOptions::default()
        .module::<klaver_http::Module>()
        .search_path(".")
        .build()
        .await?;

    klaver::async_with!(vm => |ctx| {

        ctx.eval_promise(include_str!("test.js")).catch(&ctx)?.into_future::<()>().await.catch(&ctx)?;

        Ok(())
    })
    .await?;

    vm.idle().await?;

    // let vm = Arc::new(vm);
    // let mut handles = Vec::default();
    // for i in 0..5 {
    //     let vm = vm.clone();
    //     let handle = tokio::spawn(async move {
    //         klaver::async_with!(vm => |ctx| {

    //           let loaded = ctx.eval_promise(include_str!("test.js")).unwrap().into_future::<()>().await;

    //           Ok(())
    //         }).await
    //     });

    //     handles.push(handle);
    // }

    // for h in handles {
    //     h.await.unwrap().unwrap();
    // }

    Ok(())
}
