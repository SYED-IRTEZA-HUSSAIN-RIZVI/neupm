use anyhow::Result;
use clap::Parser;
use neupm::cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    neupm::run(cli).await
}
