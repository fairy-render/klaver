use rquickjs::{CatchResultExt, Module};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let vm = klaver::Options::default()
        .module::<klaver_http::Module>()
        .search_path(".")
        .build()
        .await
        .unwrap();

    klaver::async_with!(vm =>|ctx| {
        //

        Module::import(&ctx, "./klaver-http/examples/test.js").catch(&ctx)?.into_future::<()>().await.catch(&ctx)?;

        Ok(())
    }).await.unwrap();
}
