use std::path::PathBuf;

use clap::{builder::PossibleValue, Parser, Subcommand, ValueEnum};
use klaver::{
    modules::transformer::{
        swc::{CompilerOptions, Decorators},
        SwcTranspiler, Transpiler,
    },
    Options, Vm, WinterCG,
};
use rquickjs::{CatchResultExt, Module};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Clone)]
struct Deco(Decorators);

impl Default for Deco {
    fn default() -> Self {
        Deco(Decorators::Stage2022)
    }
}

impl ValueEnum for Deco {
    fn value_variants<'a>() -> &'a [Self] {
        &[Deco(Decorators::Legacy), Deco(Decorators::Stage2022)]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self.0 {
            Decorators::Legacy => Some(PossibleValue::new("legacy")),
            Decorators::Stage2022 => Some(PossibleValue::new("staging")),
        }
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    path: Option<PathBuf>,
    #[arg(short, long, default_value = "staging")]
    decorators: Deco,
    /// Enable explicit resource management
    #[arg(short, long, default_value_t = false)]
    erm: bool,
    /// Enable async context tracking
    #[arg(short, long, default_value_t = false)]
    ac: bool,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile scripts
    Compile { path: PathBuf },
    /// Generate types for registered modules
    Typings { path: Option<PathBuf> },
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    let opts = CompilerOptions {
        decorators: cli.decorators.0,
        async_context: cli.ac,
        explicit_resource_management: cli.erm,
    };

    match cli.command {
        Some(Commands::Compile { path }) => {
            compile(path, opts).await?;
        }
        Some(Commands::Typings { path }) => {
            let root = path.unwrap_or_else(|| PathBuf::from("@types"));

            let env = create_vm(opts).build_environ();
            let files = env.typings().files();
            for file in files {
                let path = file.path.to_logical_path(&root);
                if let Some(parent) = path.parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }

                tokio::fs::write(path, file.content).await?;
            }
        }
        None => {
            let Some(path) = cli.path else { return Ok(()) };
            // let args = std::env::args().skip(1).collect::<Vec<_>>();

            // if args.is_empty() {
            //     eprintln!("Usage: {} <file>", std::env::args().next().unwrap());
            //     return Ok(());
            // }

            run(path, opts).await?;
        }
    }

    Ok(())
}

fn create_vm(opts: CompilerOptions) -> Options {
    let vm = Vm::new()
        .search_path(".")
        .transpiler(SwcTranspiler::new_with(opts))
        .module::<klaver_dom::Module>()
        .module::<klaver_handlebars::Module>()
        .module::<klaver_image::Module>()
        .module::<klaver_fs::Module>();
    // .module::<klaver_http::Module>();
    vm
}

async fn run(path: PathBuf, options: CompilerOptions) -> color_eyre::Result<()> {
    let vm = create_vm(options).build().await?;

    let mut filename = path.display().to_string();

    if filename.starts_with("/") {
        eprintln!("Path should be relative to current directory");
        return Ok(());
    }

    if !filename.starts_with("./") {
        filename = format!("./{filename}");
    }

    klaver::async_with!(vm => |ctx| {

        let config = WinterCG::get(&ctx).catch(&ctx)?;
        config.borrow().init_env_from_os(ctx.clone()).catch(&ctx)?;

        Module::import(&ctx, &*filename).catch(&ctx)?.into_future::<()>().await.catch(&ctx)?;

        Ok(())
    })
    .await?;

    vm.idle().await?;

    Ok(())
}

async fn compile(path: PathBuf, options: CompilerOptions) -> color_eyre::Result<()> {
    let compiler = SwcTranspiler::new_with(options);

    let ret = compiler.compile(&path)?;

    println!("{}", ret);

    Ok(())
}
