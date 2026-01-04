#!/bin/bash
# pvm.sh - Shell wrapper for PVM (Python Version Manager)
# Source this file in your shell config: source ~/.pvm/pvm.sh

# PVM home directory
export PVM_HOME="${PVM_HOME:-$HOME/.pvm}"

# PVM binary path
PVM_BIN="$PVM_HOME/bin/pvm"

# Main pvm function
pvm() {
    case "$1 $2" in
        "env activate"|"env act")
            shift 2
            local script
            script=$("$PVM_BIN" env activation-script "$@" 2>&1)
            if [ $? -eq 0 ] && [ -f "$script" ]; then
                source "$script"

                # Set up pip wrapper for deduplication
                _pvm_setup_pip_wrapper
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

# Shell completion
if [ -n "$BASH_VERSION" ]; then
    _pvm_completions() {
        local cur="${COMP_WORDS[COMP_CWORD]}"
        local prev="${COMP_WORDS[COMP_CWORD-1]}"
        local pprev="${COMP_WORDS[COMP_CWORD-2]}"

        case "$pprev $prev" in
            "pvm env")
                COMPREPLY=($(compgen -W "create remove list activate deactivate" -- "$cur"))
                ;;
            "pvm python")
                COMPREPLY=($(compgen -W "install list available remove" -- "$cur"))
                ;;
            "env create"|"env activate"|"env act"|"env remove"|"env rm")
                # Complete with environment names
                if [ -d "$PVM_HOME/envs" ]; then
                    COMPREPLY=($(compgen -W "$(ls "$PVM_HOME/envs" 2>/dev/null)" -- "$cur"))
                fi
                ;;
            *)
                case "$prev" in
                    pvm)
                        COMPREPLY=($(compgen -W "env python pip cache update" -- "$cur"))
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
                esac
                ;;
        esac
    }
    complete -F _pvm_completions pvm
    complete -F _pvm_completions act
    complete -F _pvm_completions activate
    complete -F _pvm_completions rmenv
fi

if [ -n "$ZSH_VERSION" ]; then
    _pvm() {
        local -a commands env_commands python_commands pip_commands cache_commands
        commands=(
            'env:Manage virtual environments'
            'python:Manage Python installations'
            'pip:Package management with deduplication'
            'cache:Manage package cache'
            'update:Update Python version metadata'
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

        case "$words[2]" in
            env)
                _describe 'env command' env_commands
                ;;
            python)
                _describe 'python command' python_commands
                ;;
            pip)
                _describe 'pip command' pip_commands
                ;;
            cache)
                _describe 'cache command' cache_commands
                ;;
            *)
                _describe 'command' commands
                ;;
        esac
    }
    compdef _pvm pvm
fi

# pip wrapper setup - intercepts pip install to use pvm deduplication
_pvm_setup_pip_wrapper() {
    # Save original deactivate function
    if type deactivate &>/dev/null; then
        eval "_pvm_original_deactivate() { $(declare -f deactivate | tail -n +2); }"
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
