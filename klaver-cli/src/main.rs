use std::path::{Path, PathBuf};

use klaver::{
    modules::typescript::{CompileOptions, Compiler},
    quick::{CatchResultExt, Module},
    vm::VmOptions,
};

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    path: Option<PathBuf>,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    ///Compile scripts
    Compile { path: PathBuf },
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();

    let compiler = Compiler::new();

    match cli.command {
        Some(Commands::Compile { path }) => {
            let content = tokio::fs::read_to_string(&path).await?;
            let source = compiler.compile(
                &path.display().to_string(),
                &content,
                CompileOptions {
                    jsx: true,
                    jsx_import_source: None,
                    typescript: true,
                    ts_decorators: false,
                },
            )?;
            println!("{}", source.code);
            return Ok(());
        }
        _ => {}
    }

    let vm = VmOptions::default()
        .search_path(Path::new("."))
        .module::<klaver_streams::Module>()
        .module::<klaver_os::shell::Module>()
        .module::<klaver_compat::Compat>()
        .module::<klaver_image::Module>()
        .build()
        .await?;

    klaver_compat::init(&vm).await?;

    let args = std::env::args().skip(1).collect::<Vec<_>>();

    if args.is_empty() {
        eprintln!("Usage: {} <file>", std::env::args().next().unwrap());
        return Ok(());
    }

    let content = std::fs::read_to_string(&args[0])?;

    let source = compiler.compile(
        &args[0],
        &content,
        CompileOptions {
            jsx_import_source: None,
            jsx: true,
            typescript: true,
            ts_decorators: false,
        },
    )?;

    klaver::async_with!(vm => |ctx| {

        Module::evaluate(ctx.clone(), &*args[0], source.code).catch(&ctx)?.into_future::<()>().await.catch(&ctx)?;

        Ok(())
    })
    .await?;

    vm.idle().await?;

    Ok(())
}
