use klaver::Builder;
use klaver_wintertc::CompioBackend;

#[compio::main]
async fn main() -> klaver_vm::Result<()> {
    let vm = Builder::new(CompioBackend).search_path(".").build().await?;

    vm.run_module("./klaver/examples/klaver.js").await?;

    klaver_vm::Ok(())
}
