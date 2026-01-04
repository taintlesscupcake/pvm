//! CLI commands for pvm

use clap::Subcommand;

pub mod cache;
pub mod config;
pub mod env;
pub mod pip;
pub mod python;
pub mod update;

#[derive(Subcommand)]
pub enum Commands {
    /// Manage virtual environments
    Env {
        #[command(subcommand)]
        command: env::EnvCommands,
    },
    /// Manage Python installations
    Python {
        #[command(subcommand)]
        command: python::PythonCommands,
    },
    /// Manage package cache (deduplication)
    Cache {
        #[command(subcommand)]
        command: cache::CacheCommands,
    },
    /// pip wrapper with deduplication
    Pip {
        #[command(subcommand)]
        command: pip::PipCommands,
    },
    /// Manage PVM configuration
    Config {
        #[command(subcommand)]
        command: config::ConfigCommands,
    },
    /// Update Python version metadata
    Update,
}

pub async fn execute(command: Commands) -> anyhow::Result<()> {
    match command {
        Commands::Env { command } => env::execute(command).await,
        Commands::Python { command } => python::execute(command).await,
        Commands::Cache { command } => cache::execute(command).await,
        Commands::Pip { command } => pip::execute(command).await,
        Commands::Config { command } => config::execute(command).await,
        Commands::Update => update::execute().await,
    }
}
