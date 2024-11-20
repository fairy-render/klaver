use std::path::PathBuf;

use klaver::{modules::transformer::Compiler, Options, Vm, WinterCG};

use clap::{Parser, Subcommand};
use rquickjs::{CatchResultExt, Module};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    path: Option<PathBuf>,
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

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Compile { path }) => {
            compile(path).await?;
        }
        Some(Commands::Typings { path }) => {
            let root = path.unwrap_or_else(|| PathBuf::from("@types"));

            let env = create_vm().build_environ();
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
            let args = std::env::args().skip(1).collect::<Vec<_>>();

            if args.is_empty() {
                eprintln!("Usage: {} <file>", std::env::args().next().unwrap());
                return Ok(());
            }

            run((&args[0]).into()).await?;
        }
    }

    Ok(())
}

fn create_vm() -> Options {
    let vm = Vm::new()
        .search_path(".")
        .module::<klaver_dom::Module>()
        .module::<klaver_handlebars::Module>()
        .module::<klaver_image::Module>()
        .module::<klaver_fs::Module>();
    vm
}

async fn run(path: PathBuf) -> color_eyre::Result<()> {
    let vm = create_vm().build().await?;

    let filename = path.display().to_string();
    let content = tokio::fs::read_to_string(path).await?;

    let compiler = Compiler::default();

    let content = compiler.compile(&content, &filename).unwrap();

    klaver::async_with!(vm => |ctx| {

        let config = WinterCG::get(&ctx).catch(&ctx)?;

        config.borrow().init_env_from_os(ctx.clone()).catch(&ctx)?;

        Module::evaluate(ctx.clone(), filename, content.code).catch(&ctx)?.into_future::<()>().await.catch(&ctx)?;
        Ok(())
    })
    .await?;

    Ok(())
}

async fn compile(path: PathBuf) -> color_eyre::Result<()> {
    let compiler = Compiler::default();

    let content = tokio::fs::read_to_string(&path).await?;

    let name = path.display().to_string();

    let ret = compiler.compile(&content, &name)?;

    println!("{}", ret.code);

    Ok(())
}
