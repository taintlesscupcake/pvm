//! Shell completion generation

/// Generate shell completion script
pub fn generate_completion(shell: &str) -> String {
    match shell {
        "bash" => generate_bash_completion(),
        "zsh" => generate_zsh_completion(),
        _ => String::new(),
    }
}

fn generate_bash_completion() -> String {
    // TODO: Generate bash completion using clap_complete
    r#"# pvm bash completion
_pvm_completions() {
    local cur="${COMP_WORDS[COMP_CWORD]}"
    local prev="${COMP_WORDS[COMP_CWORD-1]}"

    case "${prev}" in
        pvm)
            COMPREPLY=($(compgen -W "env python" -- "${cur}"))
            ;;
        env)
            COMPREPLY=($(compgen -W "create remove list activate deactivate" -- "${cur}"))
            ;;
        python)
            COMPREPLY=($(compgen -W "install list available" -- "${cur}"))
            ;;
        *)
            ;;
    esac
}
complete -F _pvm_completions pvm
"#.to_string()
}

fn generate_zsh_completion() -> String {
    // TODO: Generate zsh completion using clap_complete
    r#"# pvm zsh completion
#compdef pvm

_pvm() {
    local -a commands
    commands=(
        'env:Manage virtual environments'
        'python:Manage Python installations'
    )

    _describe 'command' commands
}

compdef _pvm pvm
"#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bash_completion_not_empty() {
        let completion = generate_completion("bash");
        assert!(!completion.is_empty());
        assert!(completion.contains("pvm"));
    }

    #[test]
    fn test_zsh_completion_not_empty() {
        let completion = generate_completion("zsh");
        assert!(!completion.is_empty());
        assert!(completion.contains("pvm"));
    }

    #[test]
    fn test_unknown_shell() {
        let completion = generate_completion("unknown");
        assert!(completion.is_empty());
    }
}
