//! Migration commands for importing external virtualenvs

use anyhow::{Context, Result};
use clap::Subcommand;
use console::style;
use dialoguer::{theme::ColorfulTheme, Confirm};
use indicatif::{ProgressBar, ProgressStyle};
use pvm_core::{Config, Downloader, Installer, Migrator, PipWrapper, PythonVersion, VenvManager};
use std::path::PathBuf;

/// Default source directory for virtualenvs
fn default_source_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_default()
        .join(".virtualenvs")
        .join("envs")
}

#[derive(Subcommand)]
pub enum MigrateCommands {
    /// Migrate a virtual environment from external source
    Env {
        /// Name of the environment to migrate (or use --all)
        name: Option<String>,

        /// Source directory containing virtualenvs
        #[arg(short, long, default_value_os_t = default_source_dir())]
        source: PathBuf,

        /// New name for the migrated environment
        #[arg(long)]
        rename: Option<String>,

        /// Migrate all environments from source
        #[arg(long, conflicts_with = "name")]
        all: bool,

        /// Delete source environment after successful migration
        #[arg(long)]
        delete_source: bool,

        /// Skip confirmation prompts
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// List environments available for migration
    List {
        /// Source directory containing virtualenvs
        #[arg(short, long, default_value_os_t = default_source_dir())]
        source: PathBuf,
    },
}

pub async fn execute(command: MigrateCommands) -> Result<()> {
    let config = Config::load().context("Failed to load configuration")?;

    match command {
        MigrateCommands::Env {
            name,
            source,
            rename,
            all,
            delete_source,
            yes,
        } => {
            if all {
                migrate_all(&config, &source, delete_source, yes).await
            } else {
                let env_name = name.context("Environment name required (or use --all)")?;
                migrate_single(&config, &source, &env_name, rename.as_deref(), delete_source, yes)
                    .await
            }
        }
        MigrateCommands::List { source } => list_available(&config, &source),
    }
}

async fn migrate_single(
    config: &Config,
    source_dir: &PathBuf,
    name: &str,
    new_name: Option<&str>,
    delete_source: bool,
    skip_confirm: bool,
) -> Result<()> {
    let migrator = Migrator::new(config.clone());
    let source_path = source_dir.join(name);

    // Parse source environment
    let source_info = migrator
        .parse_source_env(&source_path)
        .context(format!("Failed to parse source environment '{}'", name))?;

    let target_name = new_name.unwrap_or(&source_info.name);

    // Check if target already exists
    let venv_manager = VenvManager::new(config.clone());
    if venv_manager.exists(target_name) {
        anyhow::bail!(
            "Environment '{}' already exists in pvm. Use --rename to specify a different name.",
            target_name
        );
    }

    println!(
        "{} environment '{}' (Python {})...",
        style("Migrating").cyan().bold(),
        name,
        source_info.python_version
    );

    // Check/install Python
    let python_installed =
        ensure_python_installed(config, &source_info.python_version).await?;
    if python_installed {
        println!(
            "  {} Python {}",
            style("Installed").green(),
            source_info.python_version
        );
    } else {
        println!(
            "  {} Python {} already available",
            style("Using").dim(),
            source_info.python_version
        );
    }

    // Copy environment
    println!("  {} environment files...", style("Copying").cyan());
    let target_path = migrator.target_env_path(target_name);
    migrator.copy_env_directory(&source_info.path, &target_path)?;

    // Fix symlinks, activate scripts, and pyvenv.cfg
    println!("  {} Python symlinks...", style("Fixing").cyan());
    migrator.fix_python_symlinks(&target_path, &source_info.python_version)?;
    migrator.fix_activate_scripts(&target_path, &source_info.path)?;
    migrator.update_pyvenv_cfg(&target_path, &source_info.python_version)?;

    // Run pip sync for deduplication
    println!("  {} packages...", style("Syncing").cyan());
    let mut wrapper = PipWrapper::new(target_path.clone(), config.clone())
        .context("Failed to create pip wrapper")?;
    let sync_result = wrapper.sync_all().context("Failed to sync packages")?;

    // Print results
    println!();
    println!(
        "{} Migrated '{}' to pvm",
        style("✓").green().bold(),
        target_name
    );
    println!("  Path: {}", target_path.display());
    if sync_result.from_cache > 0 {
        println!(
            "  Deduplicated: {} packages (saved {})",
            sync_result.from_cache,
            format_bytes(sync_result.saved_bytes)
        );
    }

    // Handle source deletion
    let should_delete = if delete_source {
        true
    } else if !skip_confirm {
        should_delete_source(name)?
    } else {
        false
    };

    if should_delete {
        std::fs::remove_dir_all(&source_path)?;
        println!(
            "  {} source environment from {}",
            style("Deleted").yellow(),
            source_path.display()
        );
    }

    Ok(())
}

async fn migrate_all(
    config: &Config,
    source_dir: &PathBuf,
    delete_source: bool,
    skip_confirm: bool,
) -> Result<()> {
    let migrator = Migrator::new(config.clone());
    let envs = migrator.list_source_envs(source_dir)?;

    if envs.is_empty() {
        println!("No environments found in {}", source_dir.display());
        return Ok(());
    }

    println!(
        "{} {} environments to migrate:",
        style("Found").bold(),
        envs.len()
    );
    for env in &envs {
        println!(
            "  {} {} (Python {})",
            style("•").dim(),
            style(&env.name).cyan(),
            env.python_version
        );
    }
    println!();

    if !skip_confirm {
        let confirm = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Proceed with migration?")
            .default(true)
            .interact()?;
        if !confirm {
            println!("Cancelled.");
            return Ok(());
        }
    }

    let total = envs.len();
    let mut success_count = 0;
    let mut failed_envs = Vec::new();

    for env in envs {
        println!();
        match migrate_single(config, source_dir, &env.name, None, delete_source, true).await {
            Ok(_) => success_count += 1,
            Err(e) => {
                eprintln!(
                    "{} Failed to migrate '{}': {}",
                    style("✗").red(),
                    env.name,
                    e
                );
                failed_envs.push(env.name);
            }
        }
    }

    println!();
    println!(
        "{} Migrated {}/{} environments",
        style("Done!").green().bold(),
        success_count,
        total
    );

    if !failed_envs.is_empty() {
        println!(
            "{} Failed: {}",
            style("Warning:").yellow(),
            failed_envs.join(", ")
        );
    }

    Ok(())
}

fn list_available(config: &Config, source_dir: &PathBuf) -> Result<()> {
    let migrator = Migrator::new(config.clone());
    let envs = migrator.list_source_envs(source_dir)?;

    if envs.is_empty() {
        println!("No environments found in {}", source_dir.display());
        return Ok(());
    }

    println!("{}", style("Available for migration:").bold());
    println!("  Source: {}", source_dir.display());
    println!();

    for env in envs {
        println!(
            "  {} {} (Python {})",
            style("•").green(),
            style(&env.name).cyan().bold(),
            env.python_version
        );
    }

    println!();
    println!(
        "Migrate with: {} {} {} <name>",
        style("pvm").cyan(),
        style("migrate").cyan(),
        style("env").cyan(),
    );
    println!(
        "Migrate all:  {} {} {} {}",
        style("pvm").cyan(),
        style("migrate").cyan(),
        style("env").cyan(),
        style("--all").cyan(),
    );

    Ok(())
}

async fn ensure_python_installed(config: &Config, version: &PythonVersion) -> Result<bool> {
    let mut downloader = Downloader::new(config.clone())?;
    let installer = Installer::new(config.clone());

    // Check if already installed
    let installed = downloader.list_installed()?;
    if installed.iter().any(|v| v == version) {
        return Ok(false);
    }

    // Also check if any matching major.minor version is installed
    let minor_match = installed
        .iter()
        .find(|v| v.major == version.major && v.minor == version.minor);
    if minor_match.is_some() {
        return Ok(false);
    }

    // Install Python
    println!(
        "  {} Python {}...",
        style("Installing").cyan(),
        version
    );

    // Try to find exact version first, then fall back to major.minor
    let version_spec = version.to_string();
    let available = match downloader.find_version(&version_spec).await {
        Ok(v) => v,
        Err(_) => {
            // Try with just major.minor
            let minor_spec = format!("{}.{}", version.major, version.minor);
            downloader.find_version(&minor_spec).await?
        }
    };

    // Download with spinner
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} [{elapsed_precise}] Downloading...")?
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let archive_path = downloader.download(&available).await?;
    pb.finish_and_clear();

    // Extract
    installer.install(&archive_path, &available.version)?;

    Ok(true)
}

fn should_delete_source(name: &str) -> Result<bool> {
    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Delete source environment '{}'?", name))
        .default(false)
        .interact()
        .map_err(Into::into)
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
