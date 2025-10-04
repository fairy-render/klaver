use klaver::{Options, RuntimeError};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), RuntimeError> {
    let options = klaver::pool::VmPoolOptions::from(Options::default())?;

    let pool = klaver::pool::Pool::builder(klaver::pool::Manager::new(options)?)
        .build()
        .unwrap();

    let vm = pool.get().await.unwrap();

    vm.with(|ctx| {
        //
        ctx.eval("console.log('Hello, World')")?;
        Ok(())
    })
    .await?;

    Ok(())
}
