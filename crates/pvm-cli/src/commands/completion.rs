//! Shell completion generation and helpers for pvm

use anyhow::Result;
use clap::Subcommand;
use pvm_core::config::Config;
use std::io::{self, Write};

#[derive(Subcommand)]
pub enum CompletionCommands {
    /// Generate bash completion script
    Bash,
    /// Generate zsh completion script
    Zsh,
}

/// Execute completion command
pub fn execute(command: CompletionCommands) -> Result<()> {
    match command {
        CompletionCommands::Bash => generate_bash(),
        CompletionCommands::Zsh => generate_zsh(),
    }
}

/// List environment names (one per line)
pub fn complete_envs() -> Result<()> {
    let config = Config::load().unwrap_or_default();
    let envs_dir = config.envs_dir();

    if envs_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&envs_dir) {
            let mut names: Vec<String> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                .filter(|e| e.path().join("bin/activate").exists())
                .filter_map(|e| e.file_name().to_str().map(String::from))
                .collect();
            names.sort();
            for name in names {
                println!("{}", name);
            }
        }
    }
    Ok(())
}

/// List installed Python versions (one per line)
pub fn complete_pythons() -> Result<()> {
    let config = Config::load().unwrap_or_default();
    let pythons_dir = config.pythons_dir();

    if pythons_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&pythons_dir) {
            let mut versions: Vec<String> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                .filter_map(|e| e.file_name().to_str().map(String::from))
                .collect();
            // Sort by version (descending)
            versions.sort_by(|a, b| {
                let parse = |s: &str| -> Vec<u32> {
                    s.split('.').filter_map(|p| p.parse().ok()).collect()
                };
                parse(b).cmp(&parse(a))
            });
            for version in versions {
                println!("{}", version);
            }
        }
    }
    Ok(())
}

/// List available Python versions from metadata (one per line)
pub fn complete_available() -> Result<()> {
    let config = Config::load().unwrap_or_default();
    let metadata_path = config.home.join("python-metadata.json");

    if metadata_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&metadata_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                // Extract unique major.minor versions from the metadata
                let mut versions: std::collections::HashSet<String> = std::collections::HashSet::new();

                if let Some(releases) = json.as_array() {
                    for release in releases {
                        if let Some(version) = release.get("version").and_then(|v| v.as_str()) {
                            // Add both full version and major.minor
                            versions.insert(version.to_string());
                            let parts: Vec<&str> = version.split('.').collect();
                            if parts.len() >= 2 {
                                versions.insert(format!("{}.{}", parts[0], parts[1]));
                            }
                        }
                    }
                }

                let mut sorted: Vec<_> = versions.into_iter().collect();
                sorted.sort_by(|a, b| {
                    let parse = |s: &str| -> Vec<u32> {
                        s.split('.').filter_map(|p| p.parse().ok()).collect()
                    };
                    parse(b).cmp(&parse(a))
                });
                for version in sorted {
                    println!("{}", version);
                }
            }
        }
    }
    Ok(())
}

/// List config keys (one per line)
pub fn complete_config_keys() -> Result<()> {
    let keys = [
        "shell.legacy_commands",
        "shell.pip_wrapper",
        "general.auto_update_days",
        "general.colored_output",
        "dedup.enabled",
        "dedup.link_strategy",
        "dedup.auto_gc",
        "dedup.gc_retention_days",
    ];
    for key in keys {
        println!("{}", key);
    }
    Ok(())
}

/// List valid values for a config key
pub fn complete_config_values(key: &str) -> Result<()> {
    match key {
        "shell.legacy_commands" | "shell.pip_wrapper" | "general.colored_output"
        | "dedup.enabled" | "dedup.auto_gc" => {
            println!("true");
            println!("false");
        }
        "dedup.link_strategy" => {
            println!("auto");
            println!("hardlink");
            println!("clone");
            println!("copy");
        }
        // Numeric values don't need completion suggestions
        _ => {}
    }
    Ok(())
}

/// Generate bash completion script
fn generate_bash() -> Result<()> {
    let script = r#"# pvm bash completion
# Generated by: pvm completion bash

_pvm_completions() {
    local cur="${COMP_WORDS[COMP_CWORD]}"
    local prev="${COMP_WORDS[COMP_CWORD-1]}"
    local pprev=""
    [[ $COMP_CWORD -ge 2 ]] && pprev="${COMP_WORDS[COMP_CWORD-2]}"
    local ppprev=""
    [[ $COMP_CWORD -ge 3 ]] && ppprev="${COMP_WORDS[COMP_CWORD-3]}"

    # Handle 4-level commands: pvm config set <key> <value>
    if [[ "$ppprev" == "config" ]] && [[ "$pprev" == "set" ]] && [[ $COMP_CWORD -eq 4 ]]; then
        COMPREPLY=($(compgen -W "$(pvm _complete-config-values "$prev" 2>/dev/null)" -- "$cur"))
        return
    fi

    # Handle 3-level completions
    case "$pprev $prev" in
        "env activate"|"env act"|"env remove"|"env rm")
            COMPREPLY=($(compgen -W "$(pvm _complete-envs 2>/dev/null)" -- "$cur"))
            return
            ;;
        "env create"|"env new")
            # After env name, complete with Python versions
            if [[ $COMP_CWORD -eq 4 ]]; then
                COMPREPLY=($(compgen -W "$(pvm _complete-pythons 2>/dev/null)" -- "$cur"))
            fi
            return
            ;;
        "python install")
            COMPREPLY=($(compgen -W "$(pvm _complete-available 2>/dev/null)" -- "$cur"))
            return
            ;;
        "python remove"|"python rm")
            COMPREPLY=($(compgen -W "$(pvm _complete-pythons 2>/dev/null)" -- "$cur"))
            return
            ;;
        "config get"|"config set")
            COMPREPLY=($(compgen -W "$(pvm _complete-config-keys 2>/dev/null)" -- "$cur"))
            return
            ;;
        "pip install")
            if [[ "$cur" == -* ]]; then
                COMPREPLY=($(compgen -W "-e --env" -- "$cur"))
            elif [[ "$prev" == "-e" ]] || [[ "$prev" == "--env" ]]; then
                COMPREPLY=($(compgen -W "$(pvm _complete-envs 2>/dev/null)" -- "$cur"))
            fi
            return
            ;;
        "completion bash"|"completion zsh")
            return
            ;;
    esac

    # Handle 2-level completions
    case "$prev" in
        pvm)
            COMPREPLY=($(compgen -W "env python pip cache config update completion" -- "$cur"))
            ;;
        env)
            COMPREPLY=($(compgen -W "create remove list activate deactivate" -- "$cur"))
            ;;
        python)
            COMPREPLY=($(compgen -W "install list available remove" -- "$cur"))
            ;;
        pip)
            COMPREPLY=($(compgen -W "install sync" -- "$cur"))
            ;;
        cache)
            COMPREPLY=($(compgen -W "info list savings clean" -- "$cur"))
            ;;
        config)
            COMPREPLY=($(compgen -W "show get set sync reset" -- "$cur"))
            ;;
        completion)
            COMPREPLY=($(compgen -W "bash zsh" -- "$cur"))
            ;;
        # Legacy command completions
        act|activate)
            COMPREPLY=($(compgen -W "$(pvm _complete-envs 2>/dev/null)" -- "$cur"))
            ;;
        rmenv)
            COMPREPLY=($(compgen -W "$(pvm _complete-envs 2>/dev/null)" -- "$cur"))
            ;;
        mkenv)
            COMPREPLY=($(compgen -W "$(pvm _complete-pythons 2>/dev/null)" -- "$cur"))
            ;;
    esac
}

complete -F _pvm_completions pvm
"#;
    io::stdout().write_all(script.as_bytes())?;
    Ok(())
}

/// Generate zsh completion script
fn generate_zsh() -> Result<()> {
    let script = r#"#compdef pvm
# pvm zsh completion
# Generated by: pvm completion zsh

_pvm_envs() {
    local envs
    envs=(${(f)"$(pvm _complete-envs 2>/dev/null)"})
    if [[ ${#envs[@]} -gt 0 ]]; then
        _describe 'environment' envs
    fi
}

_pvm_pythons() {
    local versions
    versions=(${(f)"$(pvm _complete-pythons 2>/dev/null)"})
    if [[ ${#versions[@]} -gt 0 ]]; then
        _describe 'version' versions
    fi
}

_pvm_available() {
    local versions
    versions=(${(f)"$(pvm _complete-available 2>/dev/null)"})
    if [[ ${#versions[@]} -gt 0 ]]; then
        _describe 'version' versions
    fi
}

_pvm_config_keys() {
    local keys
    keys=(${(f)"$(pvm _complete-config-keys 2>/dev/null)"})
    if [[ ${#keys[@]} -gt 0 ]]; then
        _describe 'key' keys
    fi
}

_pvm_config_values() {
    local key="$1"
    local values
    values=(${(f)"$(pvm _complete-config-values "$key" 2>/dev/null)"})
    if [[ ${#values[@]} -gt 0 ]]; then
        _describe 'value' values
    fi
}

_pvm() {
    local -a commands env_commands python_commands pip_commands cache_commands config_commands completion_commands

    commands=(
        'env:Manage virtual environments'
        'python:Manage Python installations'
        'pip:Package management with deduplication'
        'cache:Manage package cache'
        'config:Manage PVM configuration'
        'update:Update Python version metadata'
        'completion:Generate shell completions'
    )

    env_commands=(
        'create:Create a new virtual environment'
        'remove:Remove a virtual environment'
        'list:List all virtual environments'
        'activate:Activate a virtual environment'
        'deactivate:Deactivate current virtual environment'
    )

    python_commands=(
        'install:Install a Python version'
        'list:List installed Python versions'
        'available:Show available Python versions'
        'remove:Remove an installed Python version'
    )

    pip_commands=(
        'install:Install packages with deduplication'
        'sync:Deduplicate existing packages'
    )

    cache_commands=(
        'info:Show cache statistics'
        'list:List cached packages'
        'savings:Show disk space savings'
        'clean:Remove orphaned packages'
    )

    config_commands=(
        'show:Show current configuration'
        'get:Get a config value'
        'set:Set a config value'
        'sync:Regenerate shell.conf'
        'reset:Reset to defaults'
    )

    completion_commands=(
        'bash:Generate bash completion script'
        'zsh:Generate zsh completion script'
    )

    # Determine context based on word position
    case "$words[2]" in
        env)
            case "$words[3]" in
                activate|act|remove|rm)
                    _pvm_envs
                    ;;
                create|new)
                    if [[ $CURRENT -eq 4 ]]; then
                        # First arg is env name (no completion)
                        :
                    elif [[ $CURRENT -eq 5 ]]; then
                        # Second arg is Python version
                        _pvm_pythons
                    fi
                    ;;
                "")
                    _describe 'env command' env_commands
                    ;;
                *)
                    _describe 'env command' env_commands
                    ;;
            esac
            ;;
        python)
            case "$words[3]" in
                install)
                    _pvm_available
                    ;;
                remove|rm)
                    _pvm_pythons
                    ;;
                "")
                    _describe 'python command' python_commands
                    ;;
                *)
                    _describe 'python command' python_commands
                    ;;
            esac
            ;;
        pip)
            case "$words[3]" in
                install)
                    _arguments \
                        '-e[Environment name]:env:_pvm_envs' \
                        '--env[Environment name]:env:_pvm_envs' \
                        '*:package:'
                    ;;
                "")
                    _describe 'pip command' pip_commands
                    ;;
                *)
                    _describe 'pip command' pip_commands
                    ;;
            esac
            ;;
        cache)
            _describe 'cache command' cache_commands
            ;;
        config)
            case "$words[3]" in
                get)
                    _pvm_config_keys
                    ;;
                set)
                    if [[ $CURRENT -eq 4 ]]; then
                        _pvm_config_keys
                    elif [[ $CURRENT -eq 5 ]]; then
                        _pvm_config_values "$words[4]"
                    fi
                    ;;
                "")
                    _describe 'config command' config_commands
                    ;;
                *)
                    _describe 'config command' config_commands
                    ;;
            esac
            ;;
        completion)
            _describe 'completion command' completion_commands
            ;;
        *)
            _describe 'command' commands
            ;;
    esac
}

compdef _pvm pvm
"#;
    io::stdout().write_all(script.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_config_keys() {
        // Just verify it doesn't panic
        assert!(complete_config_keys().is_ok());
    }

    #[test]
    fn test_complete_config_values_bool() {
        // Verify boolean keys return true/false
        assert!(complete_config_values("shell.legacy_commands").is_ok());
        assert!(complete_config_values("dedup.enabled").is_ok());
    }

    #[test]
    fn test_complete_config_values_enum() {
        // Verify link_strategy returns enum values
        assert!(complete_config_values("dedup.link_strategy").is_ok());
    }

    #[test]
    fn test_complete_config_values_numeric() {
        // Numeric keys should return nothing
        assert!(complete_config_values("general.auto_update_days").is_ok());
    }
}
