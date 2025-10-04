use klaver::Builder;

#[tokio::main(flavor = "current_thread")]
async fn main() -> klaver_vm::Result<()> {
    let vm = Builder::default().build().await?;

    vm.run_module("./klaver/examples/klaver.js").await?;

    // klaver_vm::async_with!(vm => |ctx| {
    //   ctx.eval_promise(include_str!("./klaver.js"))
    //         .catch(&ctx)?
    //         .into_future::<()>()
    //         .await.catch(&ctx)?;

    //     klaver_vm::Ok(())
    // })
    // .await?;

    klaver_vm::Ok(())
}
