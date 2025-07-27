use crate::cli::Cli;

mod cli;
mod repl;
mod run;

#[tokio::main(flavor = "current_thread")]
async fn main() -> color_eyre::Result<()> {
    Cli::run().await
}
