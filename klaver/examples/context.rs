use klaver::Builder;
use klaver_wintertc::CompioBackend;

#[compio::main]
async fn main() -> klaver_vm::Result<()> {
    let vm = Builder::new(CompioBackend).search_path(".").build().await?;

    std::fs::write("tmp.js", "console.log('Hello')").unwrap();

    let context = vm.create_context().await?;

    context.run_module("./tmp.js").await?;

    std::fs::write("tmp.js", "console.log('Hello 2')").unwrap();

    vm.create_context().await?.run_module("./tmp.js").await?;

    klaver_vm::Ok(())
}
