//! Environment management commands

use anyhow::{Context, Result};
use clap::Subcommand;
use console::style;
use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use pvm_core::{Config, Downloader, Installer, PythonVersion, VenvManager};

#[derive(Subcommand)]
pub enum EnvCommands {
    /// Create a new virtual environment
    #[command(visible_alias = "new")]
    Create {
        /// Name of the environment
        name: String,
        /// Python version (e.g., 3.11, 3.12.4). If not specified, shows selection UI.
        version: Option<String>,
    },
    /// Remove a virtual environment
    #[command(visible_alias = "rm")]
    Remove {
        /// Name of the environment
        name: String,
        /// Skip confirmation
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// List all virtual environments
    #[command(visible_alias = "ls")]
    List,
    /// Activate a virtual environment (requires shell wrapper)
    #[command(visible_alias = "act")]
    Activate {
        /// Name of the environment
        name: String,
    },
    /// Deactivate the current virtual environment (requires shell wrapper)
    #[command(visible_alias = "deact")]
    Deactivate,
    /// Generate activation script (internal use)
    #[command(hide = true)]
    ActivationScript {
        /// Name of the environment
        name: String,
    },
}

pub async fn execute(command: EnvCommands) -> Result<()> {
    let config = Config::load().context("Failed to load configuration")?;

    match command {
        EnvCommands::Create { name, version } => {
            create_env(&config, &name, version.as_deref()).await
        }
        EnvCommands::Remove { name, yes } => remove_env(&config, &name, yes),
        EnvCommands::List => list_envs(&config),
        EnvCommands::Activate { name: _ } => {
            eprintln!(
                "{} 'pvm env activate' requires the shell wrapper.",
                style("Error:").red().bold()
            );
            eprintln!("Please add to your shell config:");
            eprintln!("  source ~/.pvm/pvm.sh");
            std::process::exit(1);
        }
        EnvCommands::Deactivate => {
            eprintln!(
                "{} 'pvm env deactivate' requires the shell wrapper.",
                style("Error:").red().bold()
            );
            eprintln!("Please add to your shell config:");
            eprintln!("  source ~/.pvm/pvm.sh");
            std::process::exit(1);
        }
        EnvCommands::ActivationScript { name } => activation_script(&config, &name),
    }
}

async fn create_env(config: &Config, name: &str, version: Option<&str>) -> Result<()> {
    let venv_manager = VenvManager::new(config.clone());

    // Check if already exists
    if venv_manager.exists(name) {
        anyhow::bail!("Environment '{}' already exists", name);
    }

    // Determine Python version
    let version_spec = match version {
        Some(v) => v.to_string(),
        None => select_python_version(config).await?,
    };

    println!(
        "{} environment '{}' with Python {}...",
        style("Creating").green().bold(),
        name,
        version_spec
    );

    // Get or install Python
    let mut downloader = Downloader::new(config.clone())?;
    let installer = Installer::new(config.clone());

    // Check if Python version is installed
    let installed = downloader.list_installed()?;
    let _python_version = PythonVersion::parse(&version_spec)?;

    let python_path = if let Some(v) = installed.iter().find(|v| v.matches(&version_spec)) {
        // Already installed
        installer
            .get_python_path(v)
            .ok_or_else(|| anyhow::anyhow!("Python {} not found", v))?
    } else {
        // Need to install
        println!(
            "  {} Python {}...",
            style("Installing").cyan(),
            version_spec
        );

        let available = downloader.find_version(&version_spec).await?;
        println!(
            "  {} Python {} from python-build-standalone...",
            style("Downloading").cyan(),
            available.version
        );

        let archive_path = downloader.download(&available).await?;
        println!("  {} archive...", style("Extracting").cyan());

        let install_dir = installer.install(&archive_path, &available.version)?;
        installer.python_bin_path(&install_dir)
    };

    // Create virtual environment
    println!("  {} virtual environment...", style("Creating").cyan());
    let env_path = venv_manager.create(name, &python_path)?;

    println!(
        "{} Created environment '{}' at {}",
        style("✓").green().bold(),
        name,
        env_path.display()
    );
    println!(
        "  Activate with: {} {} {}",
        style("pvm").cyan(),
        style("env").cyan(),
        style("activate").cyan(),
    );

    Ok(())
}

async fn select_python_version(config: &Config) -> Result<String> {
    let mut downloader = Downloader::new(config.clone())?;

    println!("Fetching available Python versions...");
    let available = downloader.fetch_available_versions().await?;

    if available.is_empty() {
        anyhow::bail!("No Python versions available");
    }

    // Get unique major.minor versions
    let mut versions: Vec<String> = available
        .iter()
        .map(|p| format!("{}.{}", p.version.major, p.version.minor))
        .collect();
    versions.dedup();
    versions.truncate(5); // Show top 5

    // Also show installed versions
    let installed = downloader.list_installed()?;

    let items: Vec<String> = versions
        .iter()
        .map(|v| {
            let is_installed = installed.iter().any(|iv| iv.matches(v));
            if is_installed {
                format!("{} (installed)", v)
            } else {
                v.clone()
            }
        })
        .collect();

    if items.is_empty() {
        anyhow::bail!("No Python versions available");
    }

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select Python version")
        .items(&items)
        .default(0)
        .interact()?;

    Ok(versions[selection].clone())
}

fn remove_env(config: &Config, name: &str, skip_confirm: bool) -> Result<()> {
    let venv_manager = VenvManager::new(config.clone());

    if !venv_manager.exists(name) {
        anyhow::bail!("Environment '{}' does not exist", name);
    }

    if !skip_confirm {
        let confirm = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Remove environment '{}'?", name))
            .default(false)
            .interact()?;

        if !confirm {
            println!("Cancelled.");
            return Ok(());
        }
    }

    venv_manager.remove(name)?;
    println!(
        "{} Removed environment '{}'",
        style("✓").green().bold(),
        name
    );

    Ok(())
}

fn list_envs(config: &Config) -> Result<()> {
    let venv_manager = VenvManager::new(config.clone());
    let envs = venv_manager.list()?;

    if envs.is_empty() {
        println!("No virtual environments found.");
        println!(
            "Create one with: {} {} {} <name> [version]",
            style("pvm").cyan(),
            style("env").cyan(),
            style("create").cyan(),
        );
        return Ok(());
    }

    println!("{}", style("Virtual Environments:").bold());
    for env in envs {
        let version_str = env
            .python_version
            .map(|v| format!("Python {}", v))
            .unwrap_or_else(|| "unknown".to_string());

        println!(
            "  {} {} ({})",
            style("•").green(),
            style(&env.name).cyan().bold(),
            version_str
        );
    }

    Ok(())
}

fn activation_script(config: &Config, name: &str) -> Result<()> {
    let venv_manager = VenvManager::new(config.clone());
    let script_path = venv_manager.activation_script_path(name)?;

    // Output just the path - shell wrapper will source it
    println!("{}", script_path.display());

    Ok(())
}
