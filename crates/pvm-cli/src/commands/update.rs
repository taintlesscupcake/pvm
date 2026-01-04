//! Update command - refresh Python version metadata

use anyhow::Result;
use console::style;
use pvm_core::{Config, Downloader};

/// Execute the update command
pub async fn execute() -> Result<()> {
    let config = Config::load()?;
    let mut downloader = Downloader::new(config)?;

    // Show current metadata age if exists
    if let Some(age) = downloader.metadata_age() {
        let days = age.as_secs() / 86400;
        let hours = (age.as_secs() % 86400) / 3600;
        println!(
            "Current metadata age: {} days, {} hours",
            days, hours
        );
    }

    println!("Updating Python version metadata...");

    downloader.update_metadata().await?;

    println!(
        "{} Metadata updated successfully",
        style("✓").green().bold()
    );

    // Show available versions count
    let versions = downloader.fetch_available_versions().await?;
    println!("  {} Python versions available", versions.len());

    // Show version range
    if let (Some(newest), Some(oldest)) = (versions.first(), versions.last()) {
        println!(
            "  Range: {} - {}",
            oldest.version, newest.version
        );
    }

    Ok(())
}
