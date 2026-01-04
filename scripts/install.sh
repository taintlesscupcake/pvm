#!/bin/bash
# PVM Installation Script

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

PVM_HOME="${PVM_HOME:-$HOME/.pvm}"
PVM_BIN="$PVM_HOME/bin"

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
    LATEST_URL="https://github.com/sungjin/pvm/releases/latest/download/pvm-${FULL_PLATFORM}"
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
