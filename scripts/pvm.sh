#!/bin/bash
# pvm.sh - Shell integration for PVM (Python Version Manager)
#
# Load via one of:
#   eval "$(pvm init zsh)"      # recommended (zsh)
#   eval "$(pvm init bash)"     # recommended (bash)
#   source ~/.pvm/pvm.sh        # legacy; still supported

# State directory (does NOT hold the binary; binary is on PATH)
export PVM_HOME="${PVM_HOME:-$HOME/.pvm}"

# Migration safety net: if binary isn't on PATH but exists in the default
# install location, add it. Covers users upgrading from the pre-0.2 layout
# whose rc still sources this file before ~/.local/bin is added to PATH.
if ! command -v pvm >/dev/null 2>&1 && [ -x "$HOME/.local/bin/pvm" ]; then
    export PATH="$HOME/.local/bin:$PATH"
fi

# Marker consumed by `pvm doctor` to confirm integration is loaded
export PVM_SHELL_INTEGRATION=1

# Load shell configuration (auto-generated from config.toml)
_PVM_SHELL_CONF="$PVM_HOME/shell.conf"
if [ -f "$_PVM_SHELL_CONF" ]; then
    source "$_PVM_SHELL_CONF"
fi

# Defaults if shell.conf doesn't exist
: "${PVM_LEGACY_COMMANDS:=true}"
: "${PVM_PIP_WRAPPER:=true}"

# Main pvm function — shadows the binary to intercept activate/deactivate.
# Use `command pvm` inside to call the real binary.
pvm() {
    case "$1 $2" in
        "env activate"|"env act")
            shift 2
            local script
            script=$(command pvm env activation-script "$@" 2>&1)
            if [ $? -eq 0 ] && [ -f "$script" ]; then
                source "$script"
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
            command pvm "$@"
            ;;
    esac
}

# Legacy aliases (for users migrating from virtualenv.sh)
if [ "$PVM_LEGACY_COMMANDS" = "true" ]; then
    unalias mkenv rmenv lsenv act activate deact 2>/dev/null

    function mkenv {
        if [ $# -eq 2 ]; then
            pvm env create "$2" "$1"
        else
            pvm env create "$@"
        fi
    }
    function rmenv { pvm env remove "$@"; }
    function lsenv { pvm env list "$@"; }
    function act { pvm env activate "$@"; }
    function activate { pvm env activate "$@"; }
    function deact { pvm env deactivate; }
fi

# Shell completion (bash)
if [ -n "$BASH_VERSION" ]; then
    _pvm_completions() {
        local cur="${COMP_WORDS[COMP_CWORD]}"
        local prev="${COMP_WORDS[COMP_CWORD-1]}"
        local pprev=""
        [[ $COMP_CWORD -ge 2 ]] && pprev="${COMP_WORDS[COMP_CWORD-2]}"
        local ppprev=""
        [[ $COMP_CWORD -ge 3 ]] && ppprev="${COMP_WORDS[COMP_CWORD-3]}"

        if [[ "$ppprev" == "config" ]] && [[ "$pprev" == "set" ]] && [[ $COMP_CWORD -eq 4 ]]; then
            COMPREPLY=($(compgen -W "$(command pvm _complete-config-values "$prev" 2>/dev/null)" -- "$cur"))
            return
        fi

        case "$pprev $prev" in
            "pvm env")
                COMPREPLY=($(compgen -W "create remove list activate deactivate" -- "$cur")); return ;;
            "pvm python")
                COMPREPLY=($(compgen -W "install list available remove" -- "$cur")); return ;;
            "pvm config")
                COMPREPLY=($(compgen -W "show get set sync reset" -- "$cur")); return ;;
            "pvm pip")
                COMPREPLY=($(compgen -W "install sync" -- "$cur")); return ;;
            "pvm cache")
                COMPREPLY=($(compgen -W "info list savings clean" -- "$cur")); return ;;
            "pvm completion")
                COMPREPLY=($(compgen -W "bash zsh" -- "$cur")); return ;;
            "pvm init")
                COMPREPLY=($(compgen -W "bash zsh" -- "$cur")); return ;;
            "env activate"|"env act"|"env remove"|"env rm")
                COMPREPLY=($(compgen -W "$(command pvm _complete-envs 2>/dev/null)" -- "$cur")); return ;;
            "env create"|"env new")
                if [[ $COMP_CWORD -eq 4 ]]; then
                    COMPREPLY=($(compgen -W "$(command pvm _complete-pythons 2>/dev/null)" -- "$cur"))
                fi
                return ;;
            "python install")
                COMPREPLY=($(compgen -W "$(command pvm _complete-available 2>/dev/null)" -- "$cur")); return ;;
            "python remove"|"python rm")
                COMPREPLY=($(compgen -W "$(command pvm _complete-pythons 2>/dev/null)" -- "$cur")); return ;;
            "config get"|"config set")
                COMPREPLY=($(compgen -W "$(command pvm _complete-config-keys 2>/dev/null)" -- "$cur")); return ;;
            "pip install")
                if [[ "$cur" == -* ]]; then
                    COMPREPLY=($(compgen -W "-e --env" -- "$cur"))
                elif [[ "$prev" == "-e" ]] || [[ "$prev" == "--env" ]]; then
                    COMPREPLY=($(compgen -W "$(command pvm _complete-envs 2>/dev/null)" -- "$cur"))
                fi
                return ;;
        esac

        case "$prev" in
            pvm)
                COMPREPLY=($(compgen -W "env python pip cache config update completion init doctor" -- "$cur")) ;;
            env)
                COMPREPLY=($(compgen -W "create remove list activate deactivate" -- "$cur")) ;;
            python)
                COMPREPLY=($(compgen -W "install list available remove" -- "$cur")) ;;
            pip)
                COMPREPLY=($(compgen -W "install sync" -- "$cur")) ;;
            cache)
                COMPREPLY=($(compgen -W "info list savings clean" -- "$cur")) ;;
            config)
                COMPREPLY=($(compgen -W "show get set sync reset" -- "$cur")) ;;
            completion)
                COMPREPLY=($(compgen -W "bash zsh" -- "$cur")) ;;
            init)
                COMPREPLY=($(compgen -W "bash zsh" -- "$cur")) ;;
        esac
    }
    complete -F _pvm_completions pvm

    if [ "$PVM_LEGACY_COMMANDS" = "true" ]; then
        _pvm_legacy_env_completions() {
            local cur="${COMP_WORDS[COMP_CWORD]}"
            COMPREPLY=($(compgen -W "$(command pvm _complete-envs 2>/dev/null)" -- "$cur"))
        }
        _pvm_legacy_mkenv_completions() {
            local cur="${COMP_WORDS[COMP_CWORD]}"
            if [[ $COMP_CWORD -eq 1 ]]; then
                COMPREPLY=($(compgen -W "$(command pvm _complete-pythons 2>/dev/null)" -- "$cur"))
            fi
        }
        complete -F _pvm_legacy_env_completions act
        complete -F _pvm_legacy_env_completions activate
        complete -F _pvm_legacy_env_completions rmenv
        complete -F _pvm_legacy_mkenv_completions mkenv
    fi
fi

# Shell completion (zsh)
if [ -n "$ZSH_VERSION" ]; then
    _pvm_envs() {
        local envs
        envs=(${(f)"$(command pvm _complete-envs 2>/dev/null)"})
        [[ ${#envs[@]} -gt 0 ]] && _describe 'environment' envs
    }
    _pvm_pythons() {
        local versions
        versions=(${(f)"$(command pvm _complete-pythons 2>/dev/null)"})
        [[ ${#versions[@]} -gt 0 ]] && _describe 'version' versions
    }
    _pvm_available() {
        local versions
        versions=(${(f)"$(command pvm _complete-available 2>/dev/null)"})
        [[ ${#versions[@]} -gt 0 ]] && _describe 'version' versions
    }
    _pvm_config_keys() {
        local keys
        keys=(${(f)"$(command pvm _complete-config-keys 2>/dev/null)"})
        [[ ${#keys[@]} -gt 0 ]] && _describe 'key' keys
    }
    _pvm_config_values() {
        local key="$1"
        local values
        values=(${(f)"$(command pvm _complete-config-values "$key" 2>/dev/null)"})
        [[ ${#values[@]} -gt 0 ]] && _describe 'value' values
    }

    _pvm() {
        local -a commands env_commands python_commands pip_commands cache_commands config_commands completion_commands init_commands
        commands=(
            'env:Manage virtual environments'
            'python:Manage Python installations'
            'pip:Package management with deduplication'
            'cache:Manage package cache'
            'config:Manage PVM configuration'
            'update:Update Python version metadata'
            'completion:Generate shell completions'
            'init:Print shell init script (eval this in your rc)'
            'doctor:Diagnose PVM installation and shell integration'
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
        init_commands=(
            'bash:Print bash init script'
            'zsh:Print zsh init script'
        )

        case "$words[2]" in
            env)
                case "$words[3]" in
                    activate|act|remove|rm) _pvm_envs ;;
                    create|new)
                        if [[ $CURRENT -eq 5 ]]; then _pvm_pythons; fi ;;
                    *) _describe 'env command' env_commands ;;
                esac ;;
            python)
                case "$words[3]" in
                    install) _pvm_available ;;
                    remove|rm) _pvm_pythons ;;
                    *) _describe 'python command' python_commands ;;
                esac ;;
            pip)
                case "$words[3]" in
                    install)
                        _arguments \
                            '-e[Environment name]:env:_pvm_envs' \
                            '--env[Environment name]:env:_pvm_envs' \
                            '*:package:' ;;
                    *) _describe 'pip command' pip_commands ;;
                esac ;;
            cache) _describe 'cache command' cache_commands ;;
            config)
                case "$words[3]" in
                    get) _pvm_config_keys ;;
                    set)
                        if [[ $CURRENT -eq 4 ]]; then
                            _pvm_config_keys
                        elif [[ $CURRENT -eq 5 ]]; then
                            _pvm_config_values "$words[4]"
                        fi ;;
                    *) _describe 'config command' config_commands ;;
                esac ;;
            completion) _describe 'completion command' completion_commands ;;
            init) _describe 'init command' init_commands ;;
            *) _describe 'command' commands ;;
        esac
    }
    compdef _pvm pvm

    if [ "$PVM_LEGACY_COMMANDS" = "true" ]; then
        compdef _pvm_envs act
        compdef _pvm_envs activate
        compdef _pvm_envs rmenv
        compdef _pvm_pythons mkenv
    fi
fi

# pip wrapper — intercepts `pip install` in activated envs to use pvm dedup
_pvm_setup_pip_wrapper() {
    if type deactivate &>/dev/null; then
        if [ -n "$ZSH_VERSION" ]; then
            functions[_pvm_original_deactivate]=${functions[deactivate]}
        else
            eval "_pvm_original_deactivate() { $(declare -f deactivate | tail -n +2); }"
        fi
    fi

    pip() {
        case "$1" in
            install)
                shift
                pvm pip install "$@" ;;
            *)
                command pip "$@" ;;
        esac
    }

    deactivate() {
        unset -f pip 2>/dev/null
        if type _pvm_original_deactivate &>/dev/null; then
            _pvm_original_deactivate "$@"
            unset -f _pvm_original_deactivate 2>/dev/null
        fi
    }
}
