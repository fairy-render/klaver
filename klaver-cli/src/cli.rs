use std::path::Path;

use clap::Parser;
use klaver_core::throw_if;
use klaver_modules::{Global, global_info};
use klaver_vm::RuntimeError;
use klaver_wintertc::{TokioBackend, WinterTcInstance, fs::FileSystemEntry};
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
    #[clap(short, long, default_value_t = false)]
    compile: bool,
}

impl Cli {
    pub async fn run() -> color_eyre::Result<()> {
        let cli = Cli::parse();

        let builder = klaver::Builder::new(TokioBackend)
            .search_path(".")
            .global::<CliGlobal>()
            .module::<klaver_vm::VmModule>()
            .module::<klaver_image::Module>()
            // .module::<klaver_dom::Module>()
            .module::<klaver_runtime::TaskModule>();

        let vm = builder.build().await?;

        vm.async_with(async move |ctx| {
            //
            let instance = WinterTcInstance::from_ctx(&ctx).catch(&ctx)?;

            let path = instance
                .borrow()
                .settings()
                .file_system()
                .open(".")
                .await
                .map_err(|err| RuntimeError::Custom(Box::new(err)))?;

            ctx.globals()
                .set("Fs", FileSystemEntry { path })
                .catch(&ctx)?;
            Ok(())
        })
        .await?;

        klaver_runtime::set_promise_hook(vm.runtime()).await;

        run::run(
            vm,
            cli.path.as_ref().map(|m| &**m),
            cli.exec,
            cli.types,
            cli.compile,
        )
        .await?;

        Ok(())
    }
}

pub struct CliGlobal;

impl Global for CliGlobal {
    fn define<'a, 'js: 'a>(
        &'a self,
        _ctx: rquickjs::Ctx<'js>,
    ) -> impl Future<Output = rquickjs::Result<()>> + 'a {
        async move { Ok(()) }
    }
}

global_info!("cli" @types: "declare const Fs:FileSystem;" => CliGlobal);
