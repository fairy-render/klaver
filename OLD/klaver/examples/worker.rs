use klaver::{Options, RuntimeError};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), RuntimeError> {
    let options = Options::default();

    let vm = klaver::worker::Worker::new(options.build_environ().into(), None, None).await?;

    vm.with(|ctx| {
        //
        ctx.eval("console.log('Hello, World')")?;
        Ok(())
    })
    .await?;

    Ok(())
}
