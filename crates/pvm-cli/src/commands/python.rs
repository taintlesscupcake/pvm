//! Python installation management commands

use anyhow::{Context, Result};
use clap::Subcommand;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use pvm_core::{Config, Downloader, Installer, PythonVersion};

#[derive(Subcommand)]
pub enum PythonCommands {
    /// Install a Python version
    Install {
        /// Python version to install (e.g., 3.11, 3.12.4)
        version: String,
    },
    /// List installed Python versions
    #[command(visible_alias = "ls")]
    List,
    /// Show available Python versions for download
    Available {
        /// Show all versions (including older patches)
        #[arg(long)]
        all: bool,
    },
    /// Remove an installed Python version
    #[command(visible_alias = "rm")]
    Remove {
        /// Python version to remove
        version: String,
        /// Skip confirmation
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

pub async fn execute(command: PythonCommands) -> Result<()> {
    let config = Config::load().context("Failed to load configuration")?;

    match command {
        PythonCommands::Install { version } => install_python(&config, &version).await,
        PythonCommands::List => list_installed(&config),
        PythonCommands::Available { all } => list_available(&config, all).await,
        PythonCommands::Remove { version, yes } => remove_python(&config, &version, yes),
    }
}

async fn install_python(config: &Config, version: &str) -> Result<()> {
    let mut downloader = Downloader::new(config.clone())?;
    let installer = Installer::new(config.clone());

    // Check if already installed
    let installed = downloader.list_installed()?;
    if installed.iter().any(|v| v.matches(version)) {
        println!(
            "{} Python {} is already installed",
            style("✓").green().bold(),
            version
        );
        return Ok(());
    }

    // Find matching version
    println!("Finding Python {}...", version);
    let available = downloader.find_version(version).await?;

    println!(
        "{} Python {} from python-build-standalone",
        style("Downloading").cyan().bold(),
        available.version
    );

    // Download with spinner (size unknown from metadata)
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} [{elapsed_precise}] Downloading...")?
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let archive_path = downloader.download(&available).await?;
    pb.finish_with_message("Downloaded");

    // Extract
    println!("{} archive...", style("Extracting").cyan().bold());
    let install_dir = installer.install(&archive_path, &available.version)?;

    println!(
        "{} Python {} installed at {}",
        style("✓").green().bold(),
        available.version,
        install_dir.display()
    );

    Ok(())
}

fn list_installed(config: &Config) -> Result<()> {
    let downloader = Downloader::new(config.clone())?;
    let versions = downloader.list_installed()?;

    if versions.is_empty() {
        println!("No Python versions installed.");
        println!(
            "Install one with: {} {} {} <version>",
            style("pvm").cyan(),
            style("python").cyan(),
            style("install").cyan(),
        );
        return Ok(());
    }

    println!("{}", style("Installed Python Versions:").bold());
    for version in versions {
        println!("  {} {}", style("•").green(), style(version).cyan().bold());
    }

    Ok(())
}

async fn list_available(config: &Config, show_all: bool) -> Result<()> {
    let mut downloader = Downloader::new(config.clone())?;

    println!("Fetching available versions...");
    let available = downloader.fetch_available_versions().await?;

    if available.is_empty() {
        println!("No Python versions available.");
        return Ok(());
    }

    let installed = downloader.list_installed()?;

    println!("{}", style("Available Python Versions:").bold());

    if show_all {
        // Show all versions
        for python in available.iter().take(20) {
            let is_installed = installed.contains(&python.version);
            let marker = if is_installed {
                style("✓").green().to_string()
            } else {
                style("•").dim().to_string()
            };
            let version_style = if is_installed {
                style(python.version.to_string()).green().bold()
            } else {
                style(python.version.to_string()).cyan()
            };
            println!("  {} {}", marker, version_style);
        }
        if available.len() > 20 {
            println!("  ... and {} more", available.len() - 20);
        }
    } else {
        // Show only latest patch for each minor version
        let mut seen_minors = std::collections::HashSet::new();
        for python in &available {
            let minor_key = (python.version.major, python.version.minor);
            if seen_minors.insert(minor_key) {
                let is_installed = installed.iter().any(|v| {
                    v.major == python.version.major && v.minor == python.version.minor
                });
                let marker = if is_installed {
                    style("✓").green().to_string()
                } else {
                    style("•").dim().to_string()
                };
                let version_style = if is_installed {
                    style(python.version.to_string()).green().bold()
                } else {
                    style(python.version.to_string()).cyan()
                };
                println!("  {} {} (latest)", marker, version_style);
            }
        }
    }

    println!();
    println!(
        "Install with: {} {} {} <version>",
        style("pvm").cyan(),
        style("python").cyan(),
        style("install").cyan(),
    );

    Ok(())
}

fn remove_python(config: &Config, version: &str, skip_confirm: bool) -> Result<()> {
    let installer = Installer::new(config.clone());
    let parsed_version = PythonVersion::parse(version)?;

    if !installer.is_installed(&parsed_version) {
        anyhow::bail!("Python {} is not installed", version);
    }

    if !skip_confirm {
        use dialoguer::{theme::ColorfulTheme, Confirm};
        let confirm = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Remove Python {}?", version))
            .default(false)
            .interact()?;

        if !confirm {
            println!("Cancelled.");
            return Ok(());
        }
    }

    installer.uninstall(&parsed_version)?;
    println!(
        "{} Removed Python {}",
        style("✓").green().bold(),
        version
    );

    Ok(())
}
