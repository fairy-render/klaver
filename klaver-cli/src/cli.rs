use clap::Parser;
use klaver_task::AsyncState;
use klaver_util::BasePrimordials;

use crate::{repl::ReplCmd, run};

#[derive(clap::Parser)]
pub struct Cli {
    path: Option<String>,
    #[clap(short, long, default_value_t = false)]
    exec: bool,
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    Repl(ReplCmd),
}

impl Cli {
    pub async fn run() -> color_eyre::Result<()> {
        let cli = Cli::parse();

        let mut builder = klaver_vm::Options::default()
            .search_path(".")
            .module::<klaver_vm::VmModule>()
            .global::<klaver_wintertc::WinterCG>();

        #[cfg(feature = "swc")]
        {
            let opts = klaver_modules::transformer::swc::CompilerOptions {
                decorators: klaver_modules::transformer::swc::Decorators::Legacy,
                async_context: false,
                explicit_resource_management: false,
            };
            builder =
                builder.transpiler(klaver_modules::transformer::SwcTranspiler::new_with(opts));
        }

        let vm = builder.build().await?;

        vm.with(|ctx| {
            let _ = BasePrimordials::get(&ctx)?;
            let _ = AsyncState::instance(&ctx)?;
            klaver_wintertc::backend::Tokio::default().set_runtime(&ctx)?;
            Ok(())
        })
        .await?;

        if let Some(cmd) = &cli.command {
            match cmd {
                Commands::Repl(e) => {
                    e.run(vm).await?;
                }
            }
        } else {
            run::run(vm, cli.path.as_ref().map(|m| &**m), cli.exec).await?;
        }

        Ok(())
    }
}
