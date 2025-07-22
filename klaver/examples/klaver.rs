use klaver::Builder;
use rquickjs::CatchResultExt;

fn main() -> klaver_vm::Result<()> {
    futures::executor::block_on(async move {
        let vm = Builder::default().build().await?;

        klaver_vm::async_with!(vm => |ctx| {
          ctx.eval_promise(include_str!("./klaver.js"))
                .catch(&ctx)?
                .into_future::<()>()
                .await?;

            klaver_vm::Ok(())
        })
        .await?;

        klaver_vm::Ok(())
    })?;

    Ok(())
}
