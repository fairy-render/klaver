use std::path::PathBuf;

use klaver::{modules::transformer::Compiler, ResolveOptions, RuntimeError, Vm};

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
async fn main() -> Result<(), RuntimeError> {
    let vm = Vm::new().build().await?;

    let cli = Cli::parse();

    let compiler = Compiler::default();

    match cli.command {
        Some(Commands::Compile { path }) => {}
        Some(Commands::Typings { path }) => {}
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

async fn run(path: PathBuf) -> Result<(), RuntimeError> {
    let vm = Vm::new().search_path(".").build().await?;

    let filename = path.display().to_string();

    let content = tokio::fs::read_to_string(path).await.unwrap();

    let compiler = Compiler::default();

    klaver::async_with!(vm => |ctx| {
        Module::evaluate(ctx.clone(), filename, content).catch(&ctx)?.into_future::<()>().await.catch(&ctx)?;
        Ok(())
    })
    .await?;

    Ok(())
}
