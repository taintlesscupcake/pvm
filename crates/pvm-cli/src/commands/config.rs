//! Configuration management commands

use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use console::style;
use pvm_core::Config;

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show current configuration
    Show,

    /// Get a specific config value
    Get {
        /// Config key (e.g., shell.legacy_commands, general.auto_update_days)
        key: String,
    },

    /// Set a config value
    Set {
        /// Config key (e.g., shell.legacy_commands, general.auto_update_days)
        key: String,

        /// Value to set
        value: String,
    },

    /// Initialize configuration with options (used by install script)
    #[command(hide = true)]
    Init(InitArgs),

    /// Regenerate shell.conf from config.toml
    Sync,

    /// Reset configuration to defaults
    Reset {
        /// Skip confirmation
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

#[derive(Args)]
pub struct InitArgs {
    /// Enable legacy commands (mkenv, rmenv, etc.)
    #[arg(long, default_value = "true")]
    pub legacy_commands: String,

    /// Enable pip wrapper
    #[arg(long, default_value = "true")]
    pub pip_wrapper: String,

    /// Auto-update interval in days
    #[arg(long, default_value = "7")]
    pub auto_update_days: u32,

    /// Enable colored output
    #[arg(long, default_value = "true")]
    pub colored_output: String,
}

pub async fn execute(command: ConfigCommands) -> Result<()> {
    match command {
        ConfigCommands::Show => config_show(),
        ConfigCommands::Get { key } => config_get(&key),
        ConfigCommands::Set { key, value } => config_set(&key, &value),
        ConfigCommands::Init(args) => config_init(args),
        ConfigCommands::Sync => config_sync(),
        ConfigCommands::Reset { yes } => config_reset(yes),
    }
}

fn config_show() -> Result<()> {
    let config = Config::load().context("Failed to load configuration")?;
    let config_path = Config::config_path()?;

    println!("{}", style("PVM Configuration").bold());
    println!();
    println!(
        "  Config file: {}",
        style(config_path.display()).dim()
    );
    println!();

    println!("{}", style("[shell]").cyan());
    println!(
        "  legacy_commands = {}",
        format_bool(config.shell.legacy_commands)
    );
    println!(
        "  pip_wrapper     = {}",
        format_bool(config.shell.pip_wrapper)
    );
    println!();

    println!("{}", style("[general]").cyan());
    println!(
        "  auto_update_days = {}",
        if config.general.auto_update_days == 0 {
            style("disabled".to_string()).dim().to_string()
        } else {
            config.general.auto_update_days.to_string()
        }
    );
    println!(
        "  colored_output   = {}",
        format_bool(config.general.colored_output)
    );
    println!();

    println!("{}", style("[dedup]").cyan());
    println!(
        "  enabled          = {}",
        format_bool(config.dedup.enabled)
    );
    println!("  link_strategy    = {}", config.dedup.link_strategy);
    println!(
        "  auto_gc          = {}",
        format_bool(config.dedup.auto_gc)
    );
    println!(
        "  gc_retention_days = {}",
        config.dedup.gc_retention_days
    );

    Ok(())
}

fn config_get(key: &str) -> Result<()> {
    let config = Config::load().context("Failed to load configuration")?;

    let value = match key {
        "shell.legacy_commands" => config.shell.legacy_commands.to_string(),
        "shell.pip_wrapper" => config.shell.pip_wrapper.to_string(),
        "general.auto_update_days" => config.general.auto_update_days.to_string(),
        "general.colored_output" => config.general.colored_output.to_string(),
        "dedup.enabled" => config.dedup.enabled.to_string(),
        "dedup.link_strategy" => config.dedup.link_strategy.clone(),
        "dedup.auto_gc" => config.dedup.auto_gc.to_string(),
        "dedup.gc_retention_days" => config.dedup.gc_retention_days.to_string(),
        _ => {
            anyhow::bail!(
                "Unknown config key: {}\n\nAvailable keys:\n  \
                shell.legacy_commands\n  \
                shell.pip_wrapper\n  \
                general.auto_update_days\n  \
                general.colored_output\n  \
                dedup.enabled\n  \
                dedup.link_strategy\n  \
                dedup.auto_gc\n  \
                dedup.gc_retention_days",
                key
            );
        }
    };

    println!("{}", value);
    Ok(())
}

fn config_set(key: &str, value: &str) -> Result<()> {
    let mut config = Config::load().context("Failed to load configuration")?;

    match key {
        "shell.legacy_commands" => {
            config.shell.legacy_commands = parse_bool(value)?;
        }
        "shell.pip_wrapper" => {
            config.shell.pip_wrapper = parse_bool(value)?;
        }
        "general.auto_update_days" => {
            config.general.auto_update_days = value
                .parse()
                .context("Invalid value: expected a number")?;
        }
        "general.colored_output" => {
            config.general.colored_output = parse_bool(value)?;
        }
        "dedup.enabled" => {
            config.dedup.enabled = parse_bool(value)?;
        }
        "dedup.link_strategy" => {
            let valid = ["auto", "hardlink", "clone", "copy"];
            if !valid.contains(&value) {
                anyhow::bail!(
                    "Invalid link_strategy: {}. Valid values: {}",
                    value,
                    valid.join(", ")
                );
            }
            config.dedup.link_strategy = value.to_string();
        }
        "dedup.auto_gc" => {
            config.dedup.auto_gc = parse_bool(value)?;
        }
        "dedup.gc_retention_days" => {
            config.dedup.gc_retention_days = value
                .parse()
                .context("Invalid value: expected a number")?;
        }
        _ => {
            anyhow::bail!(
                "Unknown config key: {}\n\nAvailable keys:\n  \
                shell.legacy_commands\n  \
                shell.pip_wrapper\n  \
                general.auto_update_days\n  \
                general.colored_output\n  \
                dedup.enabled\n  \
                dedup.link_strategy\n  \
                dedup.auto_gc\n  \
                dedup.gc_retention_days",
                key
            );
        }
    }

    config.save().context("Failed to save configuration")?;
    config
        .sync_shell_config()
        .context("Failed to sync shell config")?;

    println!(
        "{} {} = {}",
        style("Set").green(),
        style(key).cyan(),
        value
    );
    println!();
    println!(
        "{}",
        style("Note: Restart your shell or run 'source ~/.pvm/pvm.sh' for changes to take effect.")
            .dim()
    );

    Ok(())
}

fn config_init(args: InitArgs) -> Result<()> {
    let mut config = Config::load().unwrap_or_default();

    config.shell.legacy_commands = parse_bool(&args.legacy_commands)?;
    config.shell.pip_wrapper = parse_bool(&args.pip_wrapper)?;
    config.general.auto_update_days = args.auto_update_days;
    config.general.colored_output = parse_bool(&args.colored_output)?;

    config.save().context("Failed to save configuration")?;
    config
        .sync_shell_config()
        .context("Failed to sync shell config")?;

    println!("{}", style("Configuration initialized.").green());

    Ok(())
}

fn config_sync() -> Result<()> {
    let config = Config::load().context("Failed to load configuration")?;

    config
        .sync_shell_config()
        .context("Failed to sync shell config")?;

    println!(
        "{} shell.conf regenerated from config.toml",
        style("✓").green()
    );
    println!(
        "  {}",
        style(config.shell_conf_path().display()).dim()
    );

    Ok(())
}

fn config_reset(skip_confirm: bool) -> Result<()> {
    if !skip_confirm {
        print!("Reset configuration to defaults? [y/N]: ");
        std::io::Write::flush(&mut std::io::stdout())?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }
    }

    let config = Config::default();
    config.save().context("Failed to save configuration")?;
    config
        .sync_shell_config()
        .context("Failed to sync shell config")?;

    println!("{}", style("Configuration reset to defaults.").green());

    Ok(())
}

fn parse_bool(value: &str) -> Result<bool> {
    match value.to_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        _ => anyhow::bail!(
            "Invalid boolean value: {}. Use true/false, yes/no, 1/0, or on/off",
            value
        ),
    }
}

fn format_bool(value: bool) -> String {
    if value {
        style("true").green().to_string()
    } else {
        style("false").red().to_string()
    }
}
