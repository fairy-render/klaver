use std::path::PathBuf;

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
    /// does testing things
    Compile {
        /// lists test values
        path: PathBuf,
    },
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
                    tsx: true,
                    jsx_import_source: None,
                },
            )?;
            println!("{}", source.code);
            return Ok(());
        }
        _ => {}
    }

    let vm = VmOptions::default()
        .search_path(".")
        .module::<klaver_streams::Module>()
        .module::<klaver_os::shell::Module>()
        .module::<klaver_compat::Compat>()
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
            tsx: true,
        },
    )?;

    let ret = klaver::async_with!(vm => |ctx| {

        let _ = Module::evaluate(ctx.clone(), &*args[0], source.code).catch(&ctx)?.into_future::<()>().await.catch(&ctx)?;

       Ok(())
    })
    .await?;

    vm.idle().await?;

    // if let Err(Error::Exception) = ret {
    //     context
    //         .with(|ctx| {
    //             let catch = ctx.catch();

    //             if !catch.is_null() {
    //                 println!(
    //                     "catch: {:?}",
    //                     catch.try_into_exception().unwrap().to_string()
    //                 );
    //             }

    //             rquickjs::Result::Ok(())
    //         })
    //         .await?;
    // }

    // runtime.idle().await;

    // let ret = context
    //     .with(|ctx| {
    //         let base = get_base(&ctx)?;
    //         let mut base = base.try_borrow_mut()?;

    //         base.uncaught(ctx)
    //     })
    //     .await;

    // if let Err(Error::Exception) = ret {
    //     context
    //         .with(|ctx| {
    //             let catch = ctx.catch();

    //             if !catch.is_null() {
    //                 println!(
    //                     "catch: {:?}",
    //                     catch
    //                         .try_into_exception()
    //                         .map(|m| m.to_string())
    //                         .or_else(|v| v
    //                             .try_into_string()
    //                             .map_err(|_| rquickjs::Error::new_from_js("not", "to"))
    //                             .and_then(|m| m.to_string()))
    //                         .unwrap()
    //                 );
    //             }

    //             rquickjs::Result::Ok(())
    //         })
    //         .await?;
    // }

    Ok(())
}
