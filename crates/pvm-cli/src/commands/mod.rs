//! CLI commands for pvm

use clap::Subcommand;

pub mod cache;
pub mod completion;
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
    /// Generate shell completion scripts
    Completion {
        #[command(subcommand)]
        command: completion::CompletionCommands,
    },

    // Hidden helper commands for shell completion
    #[command(hide = true, name = "_complete-envs")]
    CompleteEnvs,
    #[command(hide = true, name = "_complete-pythons")]
    CompletePythons,
    #[command(hide = true, name = "_complete-available")]
    CompleteAvailable,
    #[command(hide = true, name = "_complete-config-keys")]
    CompleteConfigKeys,
    #[command(hide = true, name = "_complete-config-values")]
    CompleteConfigValues {
        /// Config key to get values for
        key: String,
    },
}

pub async fn execute(command: Commands) -> anyhow::Result<()> {
    match command {
        Commands::Env { command } => env::execute(command).await,
        Commands::Python { command } => python::execute(command).await,
        Commands::Cache { command } => cache::execute(command).await,
        Commands::Pip { command } => pip::execute(command).await,
        Commands::Config { command } => config::execute(command).await,
        Commands::Update => update::execute().await,
        Commands::Completion { command } => completion::execute(command),
        // Hidden completion helpers
        Commands::CompleteEnvs => completion::complete_envs(),
        Commands::CompletePythons => completion::complete_pythons(),
        Commands::CompleteAvailable => completion::complete_available(),
        Commands::CompleteConfigKeys => completion::complete_config_keys(),
        Commands::CompleteConfigValues { key } => completion::complete_config_values(&key),
    }
}
