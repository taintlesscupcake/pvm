//! CLI commands for pvm

use clap::Subcommand;

pub mod env;
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
    /// Update Python version metadata
    Update,
}

pub async fn execute(command: Commands) -> anyhow::Result<()> {
    match command {
        Commands::Env { command } => env::execute(command).await,
        Commands::Python { command } => python::execute(command).await,
        Commands::Update => update::execute().await,
    }
}
