use klaver::Builder;

#[tokio::main(flavor = "current_thread")]
async fn main() -> klaver_vm::Result<()> {
    let vm = Builder::default().search_path(".").build().await?;

    vm.run_module("./klaver/examples/klaver.js").await?;

    klaver_vm::Ok(())
}
