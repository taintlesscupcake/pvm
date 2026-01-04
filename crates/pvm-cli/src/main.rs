//! pvm: Python Version Manager CLI
//!
//! A lightweight alternative to Anaconda for managing Python virtual environments.

use clap::Parser;

mod commands;

#[derive(Parser)]
#[command(name = "pvm")]
#[command(author, version, about = "Python Version Manager - Lightweight Anaconda alternative")]
struct Cli {
    #[command(subcommand)]
    command: commands::Commands,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    commands::execute(cli.command).await
}
