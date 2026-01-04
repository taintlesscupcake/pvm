//! Cache management commands

use anyhow::Result;
use clap::Subcommand;
use console::style;
use pvm_core::{Config, PackageCache};

#[derive(Subcommand)]
pub enum CacheCommands {
    /// Show cache statistics
    #[command(visible_alias = "stats")]
    Info,

    /// List cached packages
    #[command(visible_alias = "ls")]
    List {
        /// Filter by package name
        #[arg(short, long)]
        name: Option<String>,

        /// Show detailed info
        #[arg(short, long)]
        verbose: bool,
    },

    /// Clean unused packages from cache
    #[command(visible_alias = "gc")]
    Clean {
        /// Remove all packages (not just unreferenced)
        #[arg(long)]
        all: bool,

        /// Skip confirmation
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// Show disk space savings from deduplication
    Savings,
}

pub async fn execute(command: CacheCommands) -> Result<()> {
    let config = Config::load()?;

    match command {
        CacheCommands::Info => cache_info(&config),
        CacheCommands::List { name, verbose } => cache_list(&config, name, verbose),
        CacheCommands::Clean { all, yes } => cache_clean(&config, all, yes),
        CacheCommands::Savings => cache_savings(&config),
    }
}

fn cache_info(config: &Config) -> Result<()> {
    let cache = PackageCache::new(config.clone())?;
    let stats = cache.stats();

    println!("{}", style("Package Cache Statistics").bold());
    println!();
    println!("  Cached packages:  {}", stats.total_packages);
    println!("  Unique packages:  {}", stats.unique_packages);
    println!("  Cache size:       {}", format_bytes(stats.total_size_bytes));
    println!(
        "  Space saved:      {}",
        style(format_bytes(stats.saved_bytes)).green()
    );
    println!();
    println!(
        "  Cache location:   {}",
        style(cache.packages_dir().display()).dim()
    );

    Ok(())
}

fn cache_list(config: &Config, name_filter: Option<String>, verbose: bool) -> Result<()> {
    let cache = PackageCache::new(config.clone())?;
    let mut packages: Vec<_> = cache.list();

    // Filter by name if provided
    if let Some(filter) = &name_filter {
        let filter_lower = filter.to_lowercase();
        packages.retain(|p| p.id.name.to_lowercase().contains(&filter_lower));
    }

    if packages.is_empty() {
        if name_filter.is_some() {
            println!("No cached packages matching the filter.");
        } else {
            println!("No packages in cache.");
        }
        return Ok(());
    }

    // Sort by name then version
    packages.sort_by(|a, b| {
        a.id.name
            .cmp(&b.id.name)
            .then_with(|| a.id.version.cmp(&b.id.version))
    });

    println!(
        "{} ({} packages)",
        style("Cached Packages").bold(),
        packages.len()
    );
    println!();

    for pkg in packages {
        if verbose {
            println!(
                "  {} {} (py{}, {})",
                style(&pkg.id.name).cyan(),
                style(&pkg.id.version).yellow(),
                pkg.id.python_version,
                pkg.id.platform
            );
            println!("    Size:       {}", format_bytes(pkg.size_bytes));
            println!("    Files:      {}", pkg.file_count);
            println!("    References: {}", pkg.reference_count);
            println!(
                "    Cached:     {}",
                pkg.cached_at.format("%Y-%m-%d %H:%M")
            );
            println!(
                "    Last used:  {}",
                pkg.last_used.format("%Y-%m-%d %H:%M")
            );
            println!();
        } else {
            println!(
                "  {} {} ({}, refs: {})",
                style(&pkg.id.name).cyan(),
                style(&pkg.id.version).yellow(),
                format_bytes(pkg.size_bytes),
                pkg.reference_count
            );
        }
    }

    Ok(())
}

fn cache_clean(config: &Config, all: bool, yes: bool) -> Result<()> {
    let mut cache = PackageCache::new(config.clone())?;

    if all {
        // Clear entire cache
        let stats = cache.stats();

        if stats.total_packages == 0 {
            println!("Cache is already empty.");
            return Ok(());
        }

        if !yes {
            println!(
                "This will remove {} packages ({}).",
                stats.total_packages,
                format_bytes(stats.total_size_bytes)
            );
            print!("Are you sure? [y/N] ");

            use std::io::{self, Write};
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Cancelled.");
                return Ok(());
            }
        }

        let cleared = cache.clear()?;
        println!(
            "{}",
            style(format!(
                "Cleared {} packages, freed {}",
                cleared.total_packages,
                format_bytes(cleared.total_size_bytes)
            ))
            .green()
        );
    } else {
        // Only garbage collect orphaned packages
        let gc_stats = cache.garbage_collect()?;

        if gc_stats.removed_packages == 0 {
            println!("No orphaned packages to clean.");
        } else {
            println!(
                "{}",
                style(format!(
                    "Removed {} orphaned packages, freed {}",
                    gc_stats.removed_packages,
                    format_bytes(gc_stats.freed_bytes)
                ))
                .green()
            );
        }
    }

    Ok(())
}

fn cache_savings(config: &Config) -> Result<()> {
    let cache = PackageCache::new(config.clone())?;
    let stats = cache.stats();

    let total_if_no_dedup = stats.total_size_bytes + stats.saved_bytes;
    let savings_percent = if total_if_no_dedup > 0 {
        (stats.saved_bytes as f64 / total_if_no_dedup as f64) * 100.0
    } else {
        0.0
    };

    println!("{}", style("Deduplication Savings").bold());
    println!();
    println!(
        "  Without deduplication: {}",
        format_bytes(total_if_no_dedup)
    );
    println!(
        "  With deduplication:    {}",
        format_bytes(stats.total_size_bytes)
    );
    println!();
    println!(
        "  {} {}",
        style("Saved:").green().bold(),
        style(format!(
            "{} ({:.1}%)",
            format_bytes(stats.saved_bytes),
            savings_percent
        ))
        .green()
        .bold()
    );

    if stats.saved_bytes == 0 && stats.total_packages > 0 {
        println!();
        println!(
            "  {}",
            style("Tip: Install the same packages in multiple environments to see savings.")
                .dim()
        );
    }

    Ok(())
}

/// Format bytes in a human-readable way
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }
}
