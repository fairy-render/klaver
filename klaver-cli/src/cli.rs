use clap::Parser;
use klaver_core::{Exportable, Registry};
use klaver_worker::WebWorker;
use rquickjs::CatchResultExt;

use crate::run;

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    path: Option<String>,
    #[clap(short, long, default_value_t = false)]
    exec: bool,
    #[clap(short, long, default_value_t = false)]
    types: bool,
}

impl Cli {
    pub async fn run() -> color_eyre::Result<()> {
        let cli = Cli::parse();

        let builder = klaver::Builder::default()
            .search_path(".")
            .module::<klaver_vm::VmModule>()
            .module::<klaver_image::Module>()
            .module::<klaver_dom::Module>()
            .module::<klaver_fs::FsModule>()
            .module::<klaver_runtime::TaskModule>();

        let vm = builder.build().await?;

        vm.with(|ctx| {
            WebWorker::export(&ctx, &Registry::instance(&ctx)?, &ctx.globals()).catch(&ctx)?;
            Ok(())
        })
        .await?;

        klaver_runtime::set_promise_hook(vm.runtime()).await;

        run::run(vm, cli.path.as_ref().map(|m| &**m), cli.exec, cli.types).await?;

        Ok(())
    }
}
