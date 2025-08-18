use crate::cli::Cli;

mod cli;
mod run;

#[tokio::main(flavor = "current_thread")]
async fn main() -> color_eyre::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::level_filters::LevelFilter::TRACE)
        .init();
    Cli::run().await
}
