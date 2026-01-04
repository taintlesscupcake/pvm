# PVM - Python Version Manager

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

```bash
# Clone and build
git clone https://github.com/sungjin/pvm.git
cd pvm
cargo build --release

# Install
./scripts/install.sh

# Add to your shell config (~/.bashrc, ~/.zshrc)
source ~/.pvm/pvm.sh
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

### Aliases

For users migrating from other tools, legacy aliases are available:

```bash
mkenv <version> <name>    # → pvm env create <name> <version>
rmenv <name>              # → pvm env remove <name>
lsenv                     # → pvm env list
act <name>                # → pvm env activate <name>
deact                     # → pvm env deactivate
```

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

## Supported Platforms

- macOS (Apple Silicon / Intel)
- Linux (x86_64 / aarch64)

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

Set `PVM_HOME` to customize the installation directory:

```bash
export PVM_HOME=/custom/path
source ~/.pvm/pvm.sh
```

## License

MIT
