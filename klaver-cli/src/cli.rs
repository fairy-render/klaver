use clap::Parser;

use crate::run;

#[derive(clap::Parser)]
pub struct Cli {
    path: Option<String>,
    #[clap(short, long, default_value_t = false)]
    exec: bool,
}

impl Cli {
    pub async fn run() -> color_eyre::Result<()> {
        let cli = Cli::parse();

        let builder = klaver::Builder::default()
            .search_path(".")
            .module::<klaver_vm::VmModule>()
            .module::<klaver_runtime::TaskModule>();

        let vm = builder.build().await?;

        klaver_runtime::set_promise_hook(vm.runtime()).await;

        run::run(vm, cli.path.as_ref().map(|m| &**m), cli.exec).await?;

        Ok(())
    }
}
