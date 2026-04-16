# PVM - Python Version Manager

🇺🇸 [English](README.md) | 🇰🇷 [한국어](README_ko.md)

[**Documentation & landing page → pvm.sungjin.dev**](https://pvm.sungjin.dev)

A lightweight, standalone Python version and virtual environment manager written in Rust. Think Anaconda, but fast and simple.

## Why PVM?

| Feature | PVM | uv/mise | Anaconda |
|---------|-----|---------|----------|
| Shared environments | ✅ | ❌ (project-local) | ✅ |
| Package deduplication | ✅ (hardlinks) | ✅ | ❌ |
| No external dependencies | ✅ | ❌ | ❌ |
| Single binary | ✅ (2.6MB) | ✅ | ❌ |
| Fast | ✅ | ✅ | ❌ |

**Key differentiator**: One Python version can create multiple virtual environments, usable across different projects. Unlike tools that create `.venv` in each project directory, PVM manages environments centrally—just like Anaconda, but without the bloat.

## Installation

### Quick install (recommended)

Downloads the latest prebuilt binary for your platform, verifies its SHA256 checksum, and installs to `~/.pvm`.

```bash
curl -fsSL https://pvm.sungjin.dev/install.sh | bash
```

The binary is installed to `~/.local/bin/pvm` and state lives in `~/.pvm/`. Override paths with `PVM_BIN_DIR` / `PVM_HOME` if needed.

Then enable shell integration (activate/deactivate + completions + legacy aliases):

```bash
# zsh
echo 'eval "$(pvm init zsh)"' >> ~/.zshrc && eval "$(pvm init zsh)"

# bash
echo 'eval "$(pvm init bash)"' >> ~/.bashrc && eval "$(pvm init bash)"
```

If `~/.local/bin` isn't already on your `PATH`, also add `export PATH="$HOME/.local/bin:$PATH"` to your shell rc.

Verify the install is healthy:

```bash
pvm doctor
```

**Non-interactive install** (skip prompts, use defaults):

```bash
curl -fsSL https://pvm.sungjin.dev/install.sh | bash -s -- --yes
```

**Pin a specific version:**

```bash
curl -fsSL https://pvm.sungjin.dev/install.sh | PVM_VERSION=v0.1.0 bash
```

Supported platforms: macOS (Apple Silicon / Intel), Linux (x86_64 / aarch64).

### Build from source

Requires a Rust toolchain (edition 2021).

```bash
git clone https://github.com/taintlesscupcake/pvm.git
cd pvm
cargo build --release
./scripts/install.sh
eval "$(pvm init zsh)"   # or: pvm init bash
```

## Quick Start

```bash
# Install Python
pvm python install 3.12

# Create an environment
pvm env create myproject 3.12

# Activate it
pvm env activate myproject

# Install packages - pip is automatically wrapped for deduplication!
pip install requests numpy pandas

# Deactivate when done
pvm env deactivate
```

## Commands

### Python Management

```bash
pvm python install <version>    # Install Python (e.g., 3.12, 3.11.9, 3.8)
pvm python list                 # List installed versions
pvm python available            # Show available versions (3.8 - 3.14)
pvm python remove <version>     # Remove a version
pvm update                      # Refresh available Python versions metadata
```

### Environment Management

```bash
pvm env create <name> [version] # Create environment (interactive if no version)
pvm env list                    # List all environments
pvm env activate <name>         # Activate environment
pvm env deactivate              # Deactivate current environment
pvm env remove <name>           # Remove environment
```

### Package Management (with Deduplication)

When a pvm environment is activated, `pip install` is automatically wrapped to use deduplication:

```bash
# Activate environment first
pvm env activate myproject

# pip install now uses deduplication automatically!
pip install requests numpy           # → routes to pvm pip install
pip install -r requirements.txt      # Works with all pip install options

# Other pip commands work normally
pip uninstall requests               # → uses regular pip
pip freeze                           # → uses regular pip
pip list                             # → uses regular pip
```

You can also use `pvm pip` explicitly:

```bash
pvm pip install <packages>           # Install with deduplication
pvm pip install -r requirements.txt  # Supports all pip options
pvm pip sync                         # Deduplicate existing packages

# Specify environment without activating
pvm pip install -e <env> <packages>
pvm pip sync -e <env>
```

### Cache Management

```bash
pvm cache info                  # Show cache statistics
pvm cache list                  # List cached packages
pvm cache savings               # Show disk space savings
pvm cache clean                 # Remove orphaned packages
```

### Configuration

```bash
pvm config show                 # Show current configuration
pvm config get <key>            # Get a config value
pvm config set <key> <value>    # Set a config value
pvm config sync                 # Regenerate shell.conf from config.toml
pvm config reset                # Reset configuration to defaults
```

Available config keys:
- `shell.legacy_commands` - Enable legacy aliases (default: true)
- `shell.pip_wrapper` - Enable automatic pip wrapping (default: true)
- `general.auto_update_days` - Metadata auto-update interval (default: 7)
- `general.colored_output` - Enable colored output (default: true)
- `dedup.enabled` - Enable package deduplication (default: true)
- `dedup.link_strategy` - Link strategy: auto, hardlink, clone, copy (default: auto)

### Shell Completion

PVM supports tab completion for Bash and Zsh:

```bash
# Completions are automatically loaded when you source pvm.sh
source ~/.pvm/pvm.sh

# Or generate standalone completion scripts
pvm completion bash > ~/.bash_completion.d/pvm
pvm completion zsh > ~/.zfunc/_pvm
```

Supported completions:
- Commands and subcommands
- Environment names (`pvm env activate <TAB>`)
- Python versions (`pvm python install <TAB>`)
- Config keys and values (`pvm config set <TAB>`)

### Aliases

For users migrating from other tools, legacy aliases are available (can be disabled via `pvm config set shell.legacy_commands false`):

```bash
mkenv <version> <name>    # → pvm env create <name> <version>
rmenv <name>              # → pvm env remove <name>
lsenv                     # → pvm env list
act <name>                # → pvm env activate <name>
deact                     # → pvm env deactivate
```

### Migration from External Virtualenvs

PVM can import existing virtual environments from external sources (e.g., virtualenvwrapper, mise-managed virtualenvs):

```bash
# List environments available for migration
pvm migrate list
pvm migrate list --source /path/to/envs

# Migrate a single environment
pvm migrate env myenv
pvm migrate env myenv --rename new-name    # Rename during migration

# Migrate all environments at once
pvm migrate env --all

# Auto-delete source after migration
pvm migrate env myenv --delete-source

# Non-interactive mode
pvm migrate env myenv -y --delete-source
```

**What happens during migration:**
1. Detects Python version from source environment's `pyvenv.cfg`
2. Auto-installs matching Python version in pvm if not present
3. Copies environment to `~/.pvm/envs/`
4. Fixes Python symlinks to point to pvm-managed Python
5. Runs `pvm pip sync` to deduplicate packages into cache
6. Optionally deletes source environment (prompts by default)

**Default source:** `~/.virtualenvs/envs` (customizable with `--source`)

## Directory Structure

```
~/.pvm/
├── bin/pvm                 # PVM binary
├── pvm.sh                  # Shell integration
├── python-metadata.json    # Cached version metadata (auto-updated)
├── pythons/                # Installed Python versions
│   ├── 3.12.4/
│   └── 3.11.9/
├── envs/                   # Virtual environments
│   ├── myproject/
│   └── datascience/
├── packages/               # Deduplicated package cache
│   ├── metadata.json       # Cache metadata
│   └── store/              # Content-addressable storage
└── cache/                  # Download cache
```

## How It Works

1. **Python Installation**: Downloads prebuilt Python from [python-build-standalone](https://github.com/astral-sh/python-build-standalone) (maintained by Astral, creators of uv/ruff)

2. **Version Metadata**: Uses [uv's download-metadata.json](https://github.com/astral-sh/uv) to map Python versions (3.8 - 3.14) to the correct release tags. Metadata is cached locally and auto-updates every 7 days. Run `pvm update` to refresh manually.

3. **Environment Creation**: Uses Python's built-in `venv` module to create isolated environments

4. **Activation**: Shell wrapper sources the environment's activate script and wraps `pip install` to automatically use deduplication

5. **Package Deduplication**: When installing packages (via `pip install` in an activated environment or `pvm pip install`), identical packages across environments are stored once in a global cache and hardlinked to each environment's site-packages. This can save significant disk space when multiple environments share common packages like NumPy, PyTorch, etc.

### Important Note on Hardlinks

Package deduplication uses **hardlinks** to share files between the cache and environments. This means:

- **Shared inodes**: Multiple environments point to the same file on disk
- **Modification propagates**: If you manually modify a cached package file, the change affects ALL environments using that file
- **Recommended practice**: Avoid manually editing installed package files. Use `pip install --upgrade` or create a new environment if you need different versions

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Build release
cargo build --release
```

### Project Structure

```
crates/
├── pvm-core/     # Core library (downloader, installer, venv)
├── pvm-cli/      # CLI application
└── pvm-shell/    # Shell integration
```

## Configuration

Configuration is stored in `~/.pvm/config.toml`. Use `pvm config` commands to manage settings.

Set `PVM_HOME` to customize the installation directory:

```bash
export PVM_HOME=/custom/path
source ~/.pvm/pvm.sh
```

## License

MIT
