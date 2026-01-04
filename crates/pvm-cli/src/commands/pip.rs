//! pip wrapper commands with deduplication support

use anyhow::{Context, Result};
use clap::Subcommand;
use console::style;
use pvm_core::{Config, PipWrapper, VenvManager};
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum PipCommands {
    /// Install packages with deduplication
    Install {
        /// Environment name (optional if environment is activated)
        #[arg(short, long)]
        env: Option<String>,

        /// Arguments to pass to pip install (packages, -r requirements.txt, etc.)
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Sync all packages in an environment with the cache
    Sync {
        /// Environment name (optional if environment is activated)
        #[arg(short, long)]
        env: Option<String>,
    },
}

pub async fn execute(command: PipCommands) -> Result<()> {
    let config = Config::load().context("Failed to load configuration")?;

    match command {
        PipCommands::Install { env, args } => {
            let env_name = resolve_env_name(&config, env)?;
            pip_install(&config, &env_name, &args)
        }
        PipCommands::Sync { env } => {
            let env_name = resolve_env_name(&config, env)?;
            pip_sync(&config, &env_name)
        }
    }
}

/// Resolve environment name from explicit arg or VIRTUAL_ENV
fn resolve_env_name(config: &Config, env: Option<String>) -> Result<String> {
    if let Some(name) = env {
        return Ok(name);
    }

    // Try to detect from VIRTUAL_ENV environment variable
    if let Ok(venv_path) = std::env::var("VIRTUAL_ENV") {
        let venv_path = PathBuf::from(&venv_path);
        let envs_dir = config.envs_dir();

        // Check if VIRTUAL_ENV is under our envs directory
        if let Ok(relative) = venv_path.strip_prefix(&envs_dir) {
            if let Some(name) = relative.components().next() {
                if let Some(name_str) = name.as_os_str().to_str() {
                    return Ok(name_str.to_string());
                }
            }
        }

        // VIRTUAL_ENV exists but not managed by pvm
        anyhow::bail!(
            "Active environment '{}' is not managed by pvm.\n\
             Use -e <env> to specify a pvm environment.",
            venv_path.display()
        );
    }

    anyhow::bail!(
        "No environment specified and no active environment detected.\n\
         Either activate an environment first or use -e <env>."
    )
}

fn pip_install(config: &Config, env_name: &str, args: &[String]) -> Result<()> {
    // Check if environment exists
    let venv_manager = VenvManager::new(config.clone());
    if !venv_manager.exists(env_name) {
        anyhow::bail!("Environment '{}' does not exist", env_name);
    }

    let env_path = config.envs_dir().join(env_name);

    println!(
        "Installing packages in '{}' with deduplication...",
        style(env_name).cyan()
    );

    // Create pip wrapper
    let mut wrapper = PipWrapper::new(env_path, config.clone())
        .context("Failed to create pip wrapper")?;

    // Convert args to &str
    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    // Install packages
    let result = wrapper
        .install(&arg_refs)
        .context("Failed to install packages")?;

    // Show results
    println!();
    println!("{}", style("Installation complete!").green().bold());
    println!("  Packages installed: {}", result.packages_installed);
    if result.from_cache > 0 {
        println!(
            "  From cache:         {} (saved {})",
            style(result.from_cache).green(),
            format_bytes(result.saved_bytes)
        );
    }
    if result.added_to_cache > 0 {
        println!("  Added to cache:     {}", result.added_to_cache);
    }

    Ok(())
}

fn pip_sync(config: &Config, env_name: &str) -> Result<()> {
    // Check if environment exists
    let venv_manager = VenvManager::new(config.clone());
    if !venv_manager.exists(env_name) {
        anyhow::bail!("Environment '{}' does not exist", env_name);
    }

    let env_path = config.envs_dir().join(env_name);

    println!(
        "Syncing packages in '{}' with cache...",
        style(env_name).cyan()
    );

    // Create pip wrapper
    let mut wrapper = PipWrapper::new(env_path, config.clone())
        .context("Failed to create pip wrapper")?;

    // Sync all packages
    let result = wrapper.sync_all().context("Failed to sync packages")?;

    println!();
    println!("{}", style("Sync complete!").green().bold());
    println!("  Packages processed: {}", result.packages_installed);
    if result.from_cache > 0 {
        println!(
            "  Deduplicated:       {} (saved {})",
            style(result.from_cache).green(),
            format_bytes(result.saved_bytes)
        );
    }
    if result.added_to_cache > 0 {
        println!("  Added to cache:     {}", result.added_to_cache);
    }

    Ok(())
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
