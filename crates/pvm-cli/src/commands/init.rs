//! Shell init command — prints shell integration code for `eval "$(pvm init <shell>)"`

use anyhow::Result;
use clap::ValueEnum;
use std::io::{self, Write};

const SHELL_INTEGRATION: &str = include_str!("../../../../scripts/pvm.sh");

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum Shell {
    Bash,
    Zsh,
}

pub fn execute(_shell: Shell) -> Result<()> {
    // The wrapper auto-detects bash vs zsh via $BASH_VERSION / $ZSH_VERSION,
    // so both variants emit identical content today.
    io::stdout().write_all(SHELL_INTEGRATION.as_bytes())?;
    Ok(())
}
