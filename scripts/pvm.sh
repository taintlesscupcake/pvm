#!/bin/bash
# pvm.sh - Shell wrapper for PVM (Python Version Manager)
# Source this file in your shell config: source ~/.pvm/pvm.sh

# PVM home directory
export PVM_HOME="${PVM_HOME:-$HOME/.pvm}"

# PVM binary path
PVM_BIN="$PVM_HOME/bin/pvm"

# Load shell configuration (auto-generated from config.toml)
_PVM_SHELL_CONF="$PVM_HOME/shell.conf"
if [ -f "$_PVM_SHELL_CONF" ]; then
    source "$_PVM_SHELL_CONF"
fi

# Default values for backward compatibility (if shell.conf doesn't exist)
: "${PVM_LEGACY_COMMANDS:=true}"
: "${PVM_PIP_WRAPPER:=true}"

# Main pvm function
pvm() {
    case "$1 $2" in
        "env activate"|"env act")
            shift 2
            local script
            script=$("$PVM_BIN" env activation-script "$@" 2>&1)
            if [ $? -eq 0 ] && [ -f "$script" ]; then
                source "$script"

                # Set up pip wrapper for deduplication (if enabled)
                if [ "$PVM_PIP_WRAPPER" = "true" ]; then
                    _pvm_setup_pip_wrapper
                fi
            else
                echo "$script" >&2
                return 1
            fi
            ;;
        "env deactivate"|"env deact")
            if type deactivate &>/dev/null; then
                deactivate
            else
                echo "No virtual environment is currently active." >&2
                return 1
            fi
            ;;
        *)
            "$PVM_BIN" "$@"
            ;;
    esac
}

# Legacy alias support (for users migrating from virtualenv.sh)
# Only define if enabled in config
if [ "$PVM_LEGACY_COMMANDS" = "true" ]; then
    mkenv() {
        if [ $# -eq 2 ]; then
            # mkenv <version> <name> -> pvm env create <name> <version>
            pvm env create "$2" "$1"
        else
            pvm env create "$@"
        fi
    }

    rmenv() {
        pvm env remove "$@"
    }

    lsenv() {
        pvm env list "$@"
    }

    act() {
        pvm env activate "$@"
    }

    activate() {
        pvm env activate "$@"
    }

    deact() {
        pvm env deactivate
    }
fi

# Shell completion
if [ -n "$BASH_VERSION" ]; then
    _pvm_completions() {
        local cur="${COMP_WORDS[COMP_CWORD]}"
        local prev="${COMP_WORDS[COMP_CWORD-1]}"
        local pprev=""
        [[ $COMP_CWORD -ge 2 ]] && pprev="${COMP_WORDS[COMP_CWORD-2]}"
        local ppprev=""
        [[ $COMP_CWORD -ge 3 ]] && ppprev="${COMP_WORDS[COMP_CWORD-3]}"

        # Handle 4-level commands: pvm config set <key> <value>
        if [[ "$ppprev" == "config" ]] && [[ "$pprev" == "set" ]] && [[ $COMP_CWORD -eq 4 ]]; then
            COMPREPLY=($(compgen -W "$("$PVM_BIN" _complete-config-values "$prev" 2>/dev/null)" -- "$cur"))
            return
        fi

        # Handle 3-level completions
        case "$pprev $prev" in
            "pvm env")
                COMPREPLY=($(compgen -W "create remove list activate deactivate" -- "$cur"))
                return
                ;;
            "pvm python")
                COMPREPLY=($(compgen -W "install list available remove" -- "$cur"))
                return
                ;;
            "pvm config")
                COMPREPLY=($(compgen -W "show get set sync reset" -- "$cur"))
                return
                ;;
            "pvm pip")
                COMPREPLY=($(compgen -W "install sync" -- "$cur"))
                return
                ;;
            "pvm cache")
                COMPREPLY=($(compgen -W "info list savings clean" -- "$cur"))
                return
                ;;
            "pvm completion")
                COMPREPLY=($(compgen -W "bash zsh" -- "$cur"))
                return
                ;;
            "env activate"|"env act"|"env remove"|"env rm")
                # Complete with environment names using CLI helper
                COMPREPLY=($(compgen -W "$("$PVM_BIN" _complete-envs 2>/dev/null)" -- "$cur"))
                return
                ;;
            "env create"|"env new")
                # After env name, complete with Python versions
                if [[ $COMP_CWORD -eq 4 ]]; then
                    COMPREPLY=($(compgen -W "$("$PVM_BIN" _complete-pythons 2>/dev/null)" -- "$cur"))
                fi
                return
                ;;
            "python install")
                COMPREPLY=($(compgen -W "$("$PVM_BIN" _complete-available 2>/dev/null)" -- "$cur"))
                return
                ;;
            "python remove"|"python rm")
                COMPREPLY=($(compgen -W "$("$PVM_BIN" _complete-pythons 2>/dev/null)" -- "$cur"))
                return
                ;;
            "config get"|"config set")
                COMPREPLY=($(compgen -W "$("$PVM_BIN" _complete-config-keys 2>/dev/null)" -- "$cur"))
                return
                ;;
            "pip install")
                if [[ "$cur" == -* ]]; then
                    COMPREPLY=($(compgen -W "-e --env" -- "$cur"))
                elif [[ "$prev" == "-e" ]] || [[ "$prev" == "--env" ]]; then
                    COMPREPLY=($(compgen -W "$("$PVM_BIN" _complete-envs 2>/dev/null)" -- "$cur"))
                fi
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
        esac
    }
    complete -F _pvm_completions pvm

    # Register completions for legacy commands
    if [ "$PVM_LEGACY_COMMANDS" = "true" ]; then
        _pvm_legacy_env_completions() {
            local cur="${COMP_WORDS[COMP_CWORD]}"
            COMPREPLY=($(compgen -W "$("$PVM_BIN" _complete-envs 2>/dev/null)" -- "$cur"))
        }
        _pvm_legacy_mkenv_completions() {
            local cur="${COMP_WORDS[COMP_CWORD]}"
            # First arg is version
            if [[ $COMP_CWORD -eq 1 ]]; then
                COMPREPLY=($(compgen -W "$("$PVM_BIN" _complete-pythons 2>/dev/null)" -- "$cur"))
            fi
        }
        complete -F _pvm_legacy_env_completions act
        complete -F _pvm_legacy_env_completions activate
        complete -F _pvm_legacy_env_completions rmenv
        complete -F _pvm_legacy_mkenv_completions mkenv
    fi
fi

if [ -n "$ZSH_VERSION" ]; then
    # Helper functions for dynamic completion
    _pvm_envs() {
        local envs
        envs=(${(f)"$("$PVM_BIN" _complete-envs 2>/dev/null)"})
        if [[ ${#envs[@]} -gt 0 ]]; then
            _describe 'environment' envs
        fi
    }

    _pvm_pythons() {
        local versions
        versions=(${(f)"$("$PVM_BIN" _complete-pythons 2>/dev/null)"})
        if [[ ${#versions[@]} -gt 0 ]]; then
            _describe 'version' versions
        fi
    }

    _pvm_available() {
        local versions
        versions=(${(f)"$("$PVM_BIN" _complete-available 2>/dev/null)"})
        if [[ ${#versions[@]} -gt 0 ]]; then
            _describe 'version' versions
        fi
    }

    _pvm_config_keys() {
        local keys
        keys=(${(f)"$("$PVM_BIN" _complete-config-keys 2>/dev/null)"})
        if [[ ${#keys[@]} -gt 0 ]]; then
            _describe 'key' keys
        fi
    }

    _pvm_config_values() {
        local key="$1"
        local values
        values=(${(f)"$("$PVM_BIN" _complete-config-values "$key" 2>/dev/null)"})
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

    # Register completions for legacy commands
    if [ "$PVM_LEGACY_COMMANDS" = "true" ]; then
        compdef _pvm_envs act
        compdef _pvm_envs activate
        compdef _pvm_envs rmenv
        compdef _pvm_pythons mkenv
    fi
fi

# pip wrapper setup - intercepts pip install to use pvm deduplication
_pvm_setup_pip_wrapper() {
    # Save original deactivate function (handle both bash and zsh)
    if type deactivate &>/dev/null; then
        if [ -n "$ZSH_VERSION" ]; then
            # zsh: direct function copy via functions array (avoids eval issues)
            functions[_pvm_original_deactivate]=${functions[deactivate]}
        else
            # bash: use declare -f with eval
            eval "_pvm_original_deactivate() { $(declare -f deactivate | tail -n +2); }"
        fi
    fi

    # Define pip wrapper function
    pip() {
        case "$1" in
            install)
                shift
                pvm pip install "$@"
                ;;
            *)
                command pip "$@"
                ;;
        esac
    }

    # Wrap deactivate to clean up pip wrapper
    deactivate() {
        # Remove pip wrapper
        unset -f pip 2>/dev/null

        # Call original deactivate
        if type _pvm_original_deactivate &>/dev/null; then
            _pvm_original_deactivate "$@"
            unset -f _pvm_original_deactivate 2>/dev/null
        fi
    }
}

# Add pvm to PATH if not already there
if [[ ":$PATH:" != *":$PVM_HOME/bin:"* ]]; then
    export PATH="$PVM_HOME/bin:$PATH"
fi
