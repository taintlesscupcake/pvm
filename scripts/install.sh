#!/bin/bash
# PVM Installation Script

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[0;33m'
DIM='\033[0;90m'
NC='\033[0m'

PVM_HOME="${PVM_HOME:-$HOME/.pvm}"
PVM_BIN_DIR="${PVM_BIN_DIR:-$HOME/.local/bin}"

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
            echo ""
            echo "Environment variables:"
            echo "  PVM_HOME         State directory (default: \$HOME/.pvm)"
            echo "  PVM_BIN_DIR      Binary install directory (default: \$HOME/.local/bin)"
            echo "  PVM_VERSION      Release tag to install (default: latest)"
            echo "  PVM_RELEASE_URL  Override release asset base URL"
            exit 0
            ;;
    esac
done

# When piped from curl ("curl ... | bash"), stdin is the script itself, so
# prompts have to read from /dev/tty instead. We open it on fd 3 and route
# each `read` through that fd; globally swapping fd 0 would make bash start
# reading the *rest of the script* from the tty (no output, looks hung).
TTY_FD=""
if [ -t 0 ]; then
    TTY_FD=0
elif { exec 3</dev/tty; } 2>/dev/null; then
    TTY_FD=3
else
    INTERACTIVE=false
fi

echo -e "${CYAN}Installing PVM (Python Version Manager)...${NC}"

mkdir -p "$PVM_BIN_DIR"
mkdir -p "$PVM_HOME/pythons"
mkdir -p "$PVM_HOME/envs"
mkdir -p "$PVM_HOME/cache"

# Detect platform
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Darwin) PLATFORM="apple-darwin" ;;
    Linux)  PLATFORM="unknown-linux-gnu" ;;
    *)
        echo -e "${RED}Unsupported OS: $OS${NC}"
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64)       FULL_PLATFORM="${ARCH}-${PLATFORM}" ;;
    arm64|aarch64) FULL_PLATFORM="aarch64-${PLATFORM}" ;;
    *)
        echo -e "${RED}Unsupported architecture: $ARCH${NC}"
        exit 1
        ;;
esac

echo "Platform: $FULL_PLATFORM"
echo "Binary:   $PVM_BIN_DIR/pvm"
echo "State:    $PVM_HOME"

# Check if running from source directory (development install)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd 2>/dev/null)" || SCRIPT_DIR=""
SOURCE_BIN="$SCRIPT_DIR/../target/release/pvm"
SOURCE_SHELL_SCRIPT="$SCRIPT_DIR/pvm.sh"

if [ -n "$SCRIPT_DIR" ] && [ -f "$SOURCE_BIN" ]; then
    echo "Installing from local build..."
    install -m 0755 "$SOURCE_BIN" "$PVM_BIN_DIR/pvm"
    if [ -f "$SOURCE_SHELL_SCRIPT" ]; then
        install -m 0644 "$SOURCE_SHELL_SCRIPT" "$PVM_HOME/pvm.sh"
    fi
else
    # Download prebuilt release archive
    echo "Downloading PVM..."
    TMPDIR="$(mktemp -d)"
    trap 'rm -rf "$TMPDIR"' EXIT

    ARCHIVE="pvm-${FULL_PLATFORM}.tar.gz"
    if [ -n "${PVM_RELEASE_URL:-}" ]; then
        RELEASE_BASE="$PVM_RELEASE_URL"
    elif [ -n "${PVM_VERSION:-}" ]; then
        RELEASE_BASE="https://github.com/taintlesscupcake/pvm/releases/download/${PVM_VERSION}"
    else
        RELEASE_BASE="https://github.com/taintlesscupcake/pvm/releases/latest/download"
    fi
    ARCHIVE_URL="${RELEASE_BASE}/${ARCHIVE}"
    CHECKSUM_URL="${ARCHIVE_URL}.sha256"

    echo "Source: ${ARCHIVE_URL}"

    curl -fsSL "$ARCHIVE_URL" -o "$TMPDIR/$ARCHIVE" || {
        echo -e "${RED}Failed to download PVM from ${ARCHIVE_URL}${NC}"
        echo -e "${RED}Build from source instead: cargo build --release && ./scripts/install.sh${NC}"
        exit 1
    }

    if curl -fsSL "$CHECKSUM_URL" -o "$TMPDIR/$ARCHIVE.sha256" 2>/dev/null; then
        ( cd "$TMPDIR" && shasum -a 256 -c "$ARCHIVE.sha256" >/dev/null ) || {
            echo -e "${RED}Checksum verification failed for ${ARCHIVE}${NC}"
            exit 1
        }
    else
        echo -e "${YELLOW}Warning: checksum file unavailable, skipping verification${NC}"
    fi

    tar -xzf "$TMPDIR/$ARCHIVE" -C "$TMPDIR"
    EXTRACTED_DIR="$(find "$TMPDIR" -mindepth 1 -maxdepth 1 -type d -name 'pvm-*' | head -n 1)"
    if [ -z "$EXTRACTED_DIR" ] || [ ! -f "$EXTRACTED_DIR/pvm" ]; then
        echo -e "${RED}Archive did not contain expected pvm binary${NC}"
        exit 1
    fi

    install -m 0755 "$EXTRACTED_DIR/pvm" "$PVM_BIN_DIR/pvm"
    if [ -f "$EXTRACTED_DIR/pvm.sh" ]; then
        install -m 0644 "$EXTRACTED_DIR/pvm.sh" "$PVM_HOME/pvm.sh"
    fi
fi

# Fallback pvm.sh from the repo if not written above (rare)
if [ ! -f "$PVM_HOME/pvm.sh" ]; then
    curl -fsSL "https://raw.githubusercontent.com/taintlesscupcake/pvm/main/scripts/pvm.sh" \
        -o "$PVM_HOME/pvm.sh" 2>/dev/null \
        || echo -e "${YELLOW}Warning: could not fetch pvm.sh fallback${NC}"
fi

# Migrate from old binary location (~/.pvm/bin/pvm). The state dir stays put,
# we only remove the stale executable so shells don't accidentally run it.
LEGACY_BIN="$PVM_HOME/bin/pvm"
if [ -f "$LEGACY_BIN" ] && [ "$LEGACY_BIN" != "$PVM_BIN_DIR/pvm" ]; then
    echo -e "${DIM}Removing legacy binary at $LEGACY_BIN${NC}"
    rm -f "$LEGACY_BIN"
    rmdir "$PVM_HOME/bin" 2>/dev/null || true
fi

# Configuration setup
echo ""
echo -e "${CYAN}=== Configuration Setup ===${NC}"
echo ""

LEGACY_COMMANDS="true"
PIP_WRAPPER="true"
AUTO_UPDATE_DAYS="7"
COLORED_OUTPUT="true"

if [ "$INTERACTIVE" = "true" ]; then
    read_yn() {
        local prompt="$1" default="$2" result
        if [ "$default" = "Y" ]; then
            prompt="$prompt [Y/n]: "
        else
            prompt="$prompt [y/N]: "
        fi
        read -r -u "$TTY_FD" -p "$prompt" result
        result="${result:-$default}"
        case "$result" in
            [Yy]*) echo "true" ;;
            *)     echo "false" ;;
        esac
    }

    echo -e "${DIM}PVM can provide shell aliases for common operations.${NC}"
    echo ""

    echo "Enable legacy commands (mkenv, rmenv, lsenv, act, deact)?"
    echo -e "${DIM}  Shortcuts for users familiar with virtualenv.sh${NC}"
    LEGACY_COMMANDS=$(read_yn "" "Y")
    echo ""

    echo "Enable automatic pip wrapper?"
    echo -e "${DIM}  Routes 'pip install' through PVM for package deduplication${NC}"
    PIP_WRAPPER=$(read_yn "" "Y")
    echo ""

    echo "Auto-update Python metadata?"
    echo -e "${DIM}  Checks for new Python versions periodically${NC}"
    read -r -u "$TTY_FD" -p "Update interval in days (0 to disable) [7]: " AUTO_UPDATE_DAYS
    AUTO_UPDATE_DAYS="${AUTO_UPDATE_DAYS:-7}"
    echo ""

    COLORED_OUTPUT=$(read_yn "Enable colored output?" "Y")
    echo ""
else
    echo -e "${DIM}Using default configuration (--yes specified)${NC}"
    echo ""
fi

"$PVM_BIN_DIR/pvm" config init \
    --legacy-commands="$LEGACY_COMMANDS" \
    --pip-wrapper="$PIP_WRAPPER" \
    --auto-update-days="$AUTO_UPDATE_DAYS" \
    --colored-output="$COLORED_OUTPUT"

# Detect default shell for the init hint
SHELL_NAME="$(basename "${SHELL:-bash}")"
case "$SHELL_NAME" in
    zsh)  RC_HINT="~/.zshrc";  INIT_SHELL="zsh" ;;
    bash) RC_HINT="~/.bashrc"; INIT_SHELL="bash" ;;
    *)    RC_HINT="~/.${SHELL_NAME}rc"; INIT_SHELL="bash" ;;
esac

echo ""
echo -e "${GREEN}✓ PVM installed${NC}"
echo ""
echo -e "${CYAN}━━ Next steps — copy/paste into your terminal ━━${NC}"
echo ""

STEP=1

# Step 1 (conditional): add PVM_BIN_DIR to PATH
NEED_PATH=1
case ":$PATH:" in
    *":$PVM_BIN_DIR:"*) NEED_PATH=0 ;;
esac

if [ "$NEED_PATH" = "1" ]; then
    echo -e "  ${YELLOW}${STEP}.${NC} Add ${CYAN}$PVM_BIN_DIR${NC} to your PATH:"
    echo ""
    echo -e "       ${CYAN}echo 'export PATH=\"$PVM_BIN_DIR:\$PATH\"' >> $RC_HINT${NC}"
    echo ""
    STEP=$((STEP+1))
fi

# Step N: enable shell integration
echo -e "  ${YELLOW}${STEP}.${NC} Enable shell integration (activate/deactivate, completions, legacy aliases):"
echo ""
echo -e "       ${CYAN}echo 'eval \"\$(pvm init $INIT_SHELL)\"' >> $RC_HINT${NC}"
echo ""
STEP=$((STEP+1))

# Step N+1: reload shell
echo -e "  ${YELLOW}${STEP}.${NC} Reload your shell so the new config takes effect:"
echo ""
echo -e "       ${CYAN}exec $INIT_SHELL${NC}"
echo ""
STEP=$((STEP+1))

# Step N+2: verify
echo -e "  ${YELLOW}${STEP}.${NC} Verify the install:"
echo ""
echo -e "       ${CYAN}pvm doctor${NC}"
echo ""

echo -e "${DIM}First run: pvm update && pvm python install 3.12 && pvm env create myenv 3.12${NC}"
echo -e "${DIM}Legacy alternative to step 2: \`source ~/.pvm/pvm.sh\` — still supported.${NC}"
