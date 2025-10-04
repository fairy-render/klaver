use klaver_vm::{Options, RuntimeError, Vm};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), RuntimeError> {
    let vm = Options::default()
        .module::<klaver_test::TestModule>()
        .search_path(".")
        .build()
        .await?;

    vm.run_module("./klaver-test/examples/test.js").await?;

    Ok(())
}
