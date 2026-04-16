#!/bin/bash
# PVM Installation Script

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[0;33m'
DIM='\033[0;90m'
NC='\033[0m' # No Color

PVM_HOME="${PVM_HOME:-$HOME/.pvm}"
PVM_BIN="$PVM_HOME/bin"

# Parse command line arguments
INTERACTIVE=true
for arg in "$@"; do
    case "$arg" in
        --yes|-y)
            INTERACTIVE=false
            ;;
        --help|-h)
            echo "Usage: install.sh [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --yes, -y    Skip interactive prompts, use defaults"
            echo "  --help, -h   Show this help message"
            exit 0
            ;;
    esac
done

echo -e "${CYAN}Installing PVM (Python Version Manager)...${NC}"

# Create directories
mkdir -p "$PVM_BIN"
mkdir -p "$PVM_HOME/pythons"
mkdir -p "$PVM_HOME/envs"
mkdir -p "$PVM_HOME/cache"

# Detect platform
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Darwin)
        PLATFORM="apple-darwin"
        ;;
    Linux)
        PLATFORM="unknown-linux-gnu"
        ;;
    *)
        echo -e "${RED}Unsupported OS: $OS${NC}"
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64)
        FULL_PLATFORM="${ARCH}-${PLATFORM}"
        ;;
    arm64|aarch64)
        FULL_PLATFORM="aarch64-${PLATFORM}"
        ;;
    *)
        echo -e "${RED}Unsupported architecture: $ARCH${NC}"
        exit 1
        ;;
esac

echo "Platform: $FULL_PLATFORM"

# Check if running from source directory (development install)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SOURCE_BIN="$SCRIPT_DIR/../target/release/pvm"

if [ -f "$SOURCE_BIN" ]; then
    echo "Installing from local build..."
    cp "$SOURCE_BIN" "$PVM_BIN/pvm"
else
    # Download from releases
    echo "Downloading PVM..."
    LATEST_URL="https://github.com/taintlesscupcake/pvm/releases/latest/download/pvm-${FULL_PLATFORM}"
    curl -fsSL "$LATEST_URL" -o "$PVM_BIN/pvm" || {
        echo -e "${RED}Failed to download PVM. Please build from source.${NC}"
        exit 1
    }
fi

chmod +x "$PVM_BIN/pvm"

# Install shell wrapper
SHELL_SCRIPT="$SCRIPT_DIR/pvm.sh"
if [ -f "$SHELL_SCRIPT" ]; then
    cp "$SHELL_SCRIPT" "$PVM_HOME/pvm.sh"
else
    # Download shell wrapper
    curl -fsSL "https://raw.githubusercontent.com/sungjin/pvm/main/scripts/pvm.sh" -o "$PVM_HOME/pvm.sh" || {
        echo -e "${RED}Warning: Could not download shell wrapper${NC}"
    }
fi

# Configuration setup
echo ""
echo -e "${CYAN}=== Configuration Setup ===${NC}"
echo ""

# Default values
LEGACY_COMMANDS="true"
PIP_WRAPPER="true"
AUTO_UPDATE_DAYS="7"
COLORED_OUTPUT="true"

if [ "$INTERACTIVE" = "true" ]; then
    # Helper function to read yes/no with default
    read_yn() {
        local prompt="$1"
        local default="$2"
        local result

        if [ "$default" = "Y" ]; then
            prompt="$prompt [Y/n]: "
        else
            prompt="$prompt [y/N]: "
        fi

        read -r -p "$prompt" result
        result="${result:-$default}"

        case "$result" in
            [Yy]*) echo "true" ;;
            *) echo "false" ;;
        esac
    }

    echo -e "${DIM}PVM can provide shell aliases for common operations.${NC}"
    echo ""

    # Legacy commands
    echo "Enable legacy commands (mkenv, rmenv, lsenv, act, deact)?"
    echo -e "${DIM}  These provide shortcuts for users familiar with virtualenv.sh${NC}"
    LEGACY_COMMANDS=$(read_yn "" "Y")
    echo ""

    # Pip wrapper
    echo "Enable automatic pip wrapper?"
    echo -e "${DIM}  Routes 'pip install' through PVM for package deduplication${NC}"
    PIP_WRAPPER=$(read_yn "" "Y")
    echo ""

    # Auto-update interval
    echo "Auto-update Python metadata?"
    echo -e "${DIM}  Checks for new Python versions periodically${NC}"
    read -r -p "Update interval in days (0 to disable) [7]: " AUTO_UPDATE_DAYS
    AUTO_UPDATE_DAYS="${AUTO_UPDATE_DAYS:-7}"
    echo ""

    # Colored output
    COLORED_OUTPUT=$(read_yn "Enable colored output?" "Y")
    echo ""
else
    echo -e "${DIM}Using default configuration (--yes specified)${NC}"
    echo ""
fi

# Initialize configuration
"$PVM_BIN/pvm" config init \
    --legacy-commands="$LEGACY_COMMANDS" \
    --pip-wrapper="$PIP_WRAPPER" \
    --auto-update-days="$AUTO_UPDATE_DAYS" \
    --colored-output="$COLORED_OUTPUT"

echo ""
echo -e "${GREEN}PVM installed successfully!${NC}"
echo ""
echo "Add the following to your shell configuration file (~/.bashrc, ~/.zshrc, etc.):"
echo ""
echo -e "  ${CYAN}source ~/.pvm/pvm.sh${NC}"
echo ""
echo "Then restart your shell or run:"
echo ""
echo -e "  ${CYAN}source ~/.pvm/pvm.sh${NC}"
echo ""
echo "Get started:"
echo "  pvm python available    # See available Python versions"
echo "  pvm python install 3.12 # Install Python 3.12"
echo "  pvm env create myenv    # Create a virtual environment"
echo "  pvm env activate myenv  # Activate it"
