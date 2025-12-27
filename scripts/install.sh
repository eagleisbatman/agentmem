#!/usr/bin/env bash
#
# AgentMem Installer
# Usage: curl -sSL https://agentmem.dev/install.sh | bash
#

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Symbols
CHECK="${GREEN}✓${NC}"
CROSS="${RED}✗${NC}"
ARROW="${BLUE}→${NC}"

VERSION="0.1.0"
INSTALL_DIR="${HOME}/.local/bin"
CONFIG_DIR="${HOME}/.agentmem"
CREDENTIALS_FILE="${CONFIG_DIR}/credentials"

print_banner() {
    echo ""
    echo -e "${BLUE}"
    echo "    _                    _   __  __                 "
    echo "   / \   __ _  ___ _ __ | |_|  \/  | ___ _ __ ___   "
    echo "  / _ \ / _\` |/ _ \ '_ \| __| |\/| |/ _ \ '_ \` _ \  "
    echo " / ___ \ (_| |  __/ | | | |_| |  | |  __/ | | | | | "
    echo "/_/   \_\__, |\___|_| |_|\__|_|  |_|\___|_| |_| |_| "
    echo "        |___/                                       "
    echo -e "${NC}"
    echo "  Persistent memory for AI coding agents"
    echo "  Version: ${VERSION}"
    echo ""
}

detect_os() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux*)     OS_TYPE="linux";;
        Darwin*)    OS_TYPE="macos";;
        *)          echo -e "${CROSS} Unsupported OS: $OS"; exit 1;;
    esac

    case "$ARCH" in
        x86_64)     ARCH_TYPE="x86_64";;
        arm64)      ARCH_TYPE="aarch64";;
        aarch64)    ARCH_TYPE="aarch64";;
        *)          echo -e "${CROSS} Unsupported architecture: $ARCH"; exit 1;;
    esac

    echo -e "${CHECK} Detected: ${OS_TYPE} (${ARCH_TYPE})"
}

check_dependencies() {
    echo ""
    echo -e "${ARROW} Checking dependencies..."

    # Check for curl or wget
    if command -v curl &> /dev/null; then
        DOWNLOADER="curl"
        echo -e "  ${CHECK} curl installed"
    elif command -v wget &> /dev/null; then
        DOWNLOADER="wget"
        echo -e "  ${CHECK} wget installed"
    else
        echo -e "  ${CROSS} curl or wget required"
        exit 1
    fi

    # Check for Docker
    if command -v docker &> /dev/null; then
        echo -e "  ${CHECK} Docker installed"
        DOCKER_INSTALLED=true

        # Check if Docker is running
        if docker info &> /dev/null; then
            echo -e "  ${CHECK} Docker daemon running"
            DOCKER_RUNNING=true
        else
            echo -e "  ${YELLOW}!${NC} Docker installed but not running"
            DOCKER_RUNNING=false
        fi
    else
        echo -e "  ${YELLOW}!${NC} Docker not installed (required for Qdrant)"
        DOCKER_INSTALLED=false
        DOCKER_RUNNING=false
    fi
}

install_docker_prompt() {
    if [ "$DOCKER_INSTALLED" = false ]; then
        echo ""
        echo -e "${YELLOW}Docker is required for semantic search (Qdrant vector database).${NC}"
        echo ""
        read -p "Would you like instructions to install Docker? [y/N] " -n 1 -r
        echo ""

        if [[ $REPLY =~ ^[Yy]$ ]]; then
            if [ "$OS_TYPE" = "macos" ]; then
                echo ""
                echo "Install Docker Desktop for Mac:"
                echo "  1. Download from: https://www.docker.com/products/docker-desktop"
                echo "  2. Open the .dmg and drag Docker to Applications"
                echo "  3. Open Docker from Applications"
                echo "  4. Re-run this installer"
                echo ""
            else
                echo ""
                echo "Install Docker on Linux:"
                echo "  curl -fsSL https://get.docker.com | sh"
                echo "  sudo usermod -aG docker \$USER"
                echo "  newgrp docker"
                echo "  Re-run this installer"
                echo ""
            fi
            exit 0
        fi
    fi
}

create_directories() {
    echo ""
    echo -e "${ARROW} Creating directories..."

    mkdir -p "$INSTALL_DIR"
    mkdir -p "$CONFIG_DIR"

    echo -e "  ${CHECK} Created ${INSTALL_DIR}"
    echo -e "  ${CHECK} Created ${CONFIG_DIR}"
}

download_binary() {
    echo ""
    echo -e "${ARROW} Downloading AgentMem..."

    # For now, we'll build from source or use a placeholder
    # In production, this would download from GitHub releases

    BINARY_URL="https://github.com/agentmem/agentmem/releases/download/v${VERSION}/agentmem-${OS_TYPE}-${ARCH_TYPE}"
    BINARY_PATH="${INSTALL_DIR}/am"

    # Check if building from source is needed
    if [ -f "./target/release/agentmem" ]; then
        echo -e "  ${CHECK} Found local build, using that"
        cp "./target/release/agentmem" "$BINARY_PATH"
    else
        echo -e "  ${YELLOW}!${NC} Binary download not yet available"
        echo ""
        echo "  To install from source:"
        echo "    git clone https://github.com/agentmem/agentmem.git"
        echo "    cd agentmem"
        echo "    cargo build --release"
        echo "    cp target/release/agentmem ~/.local/bin/am"
        echo ""

        read -p "Build from source now? (requires Rust) [y/N] " -n 1 -r
        echo ""

        if [[ $REPLY =~ ^[Yy]$ ]]; then
            if command -v cargo &> /dev/null; then
                echo -e "${ARROW} Building from source..."
                cargo build --release
                cp "./target/release/agentmem" "$BINARY_PATH"
                echo -e "  ${CHECK} Built and installed"
            else
                echo -e "  ${CROSS} Rust not installed. Install from https://rustup.rs"
                exit 1
            fi
        else
            exit 0
        fi
    fi

    chmod +x "$BINARY_PATH"
    echo -e "  ${CHECK} Installed to ${BINARY_PATH}"
}

setup_path() {
    echo ""
    echo -e "${ARROW} Setting up PATH..."

    # Check if INSTALL_DIR is in PATH
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        SHELL_NAME=$(basename "$SHELL")

        case "$SHELL_NAME" in
            bash)
                PROFILE_FILE="$HOME/.bashrc"
                ;;
            zsh)
                PROFILE_FILE="$HOME/.zshrc"
                ;;
            *)
                PROFILE_FILE="$HOME/.profile"
                ;;
        esac

        echo "" >> "$PROFILE_FILE"
        echo "# AgentMem" >> "$PROFILE_FILE"
        echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$PROFILE_FILE"

        echo -e "  ${CHECK} Added ${INSTALL_DIR} to PATH in ${PROFILE_FILE}"
        echo -e "  ${YELLOW}!${NC} Run 'source ${PROFILE_FILE}' or restart your terminal"
    else
        echo -e "  ${CHECK} ${INSTALL_DIR} already in PATH"
    fi
}

setup_openai_key() {
    echo ""
    echo -e "${ARROW} OpenAI API Key Setup..."

    # Check if key already exists
    if [ -f "$CREDENTIALS_FILE" ]; then
        echo -e "  ${CHECK} Credentials file exists"
        return
    fi

    # Check environment variable
    if [ -n "$OPENAI_API_KEY" ]; then
        echo -e "  ${CHECK} Found OPENAI_API_KEY in environment"
        echo "OPENAI_API_KEY=$OPENAI_API_KEY" > "$CREDENTIALS_FILE"
        chmod 600 "$CREDENTIALS_FILE"
        return
    fi

    echo ""
    echo "  AgentMem uses OpenAI for embeddings and memory extraction."
    echo "  Get your API key from: https://platform.openai.com/api-keys"
    echo ""

    read -p "  Enter your OpenAI API key (or press Enter to skip): " -r OPENAI_KEY

    if [ -n "$OPENAI_KEY" ]; then
        echo "OPENAI_API_KEY=$OPENAI_KEY" > "$CREDENTIALS_FILE"
        chmod 600 "$CREDENTIALS_FILE"
        echo -e "  ${CHECK} Saved to ${CREDENTIALS_FILE}"
    else
        echo -e "  ${YELLOW}!${NC} Skipped. Set OPENAI_API_KEY later or run 'am init'"
    fi
}

start_qdrant() {
    if [ "$DOCKER_RUNNING" = true ]; then
        echo ""
        echo -e "${ARROW} Starting Qdrant vector database..."

        # Check if Qdrant is already running
        if docker ps --format '{{.Names}}' | grep -q "^agentmem-qdrant$"; then
            echo -e "  ${CHECK} Qdrant already running"
            return
        fi

        # Check if container exists but stopped
        if docker ps -a --format '{{.Names}}' | grep -q "^agentmem-qdrant$"; then
            docker start agentmem-qdrant > /dev/null
            echo -e "  ${CHECK} Started existing Qdrant container"
            return
        fi

        # Start new container
        docker run -d \
            --name agentmem-qdrant \
            -p 6333:6333 \
            -p 6334:6334 \
            -v agentmem-qdrant-data:/qdrant/storage \
            qdrant/qdrant:latest > /dev/null 2>&1

        echo -e "  ${CHECK} Started Qdrant container"

        # Wait for Qdrant to be ready
        echo -e "  ${ARROW} Waiting for Qdrant to be ready..."
        for i in {1..30}; do
            if curl -s http://localhost:6333/health > /dev/null 2>&1; then
                echo -e "  ${CHECK} Qdrant is ready"
                return
            fi
            sleep 1
        done

        echo -e "  ${YELLOW}!${NC} Qdrant may still be starting up"
    fi
}

print_success() {
    echo ""
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${GREEN}  AgentMem installed successfully!${NC}"
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo "  Next steps:"
    echo ""
    echo "    1. Initialize in your project:"
    echo "       ${BLUE}cd your-project${NC}"
    echo "       ${BLUE}am init${NC}"
    echo ""
    echo "    2. Install hooks for your AI agent:"
    echo "       ${BLUE}am hook install claude-code${NC}"
    echo ""
    echo "    3. Add your first memory:"
    echo "       ${BLUE}am mem add decision \"Use PostgreSQL\" --content \"For JSON support\"${NC}"
    echo ""
    echo "  Documentation: https://agentmem.dev/docs"
    echo "  Report issues: https://github.com/agentmem/agentmem/issues"
    echo ""
}

main() {
    print_banner
    detect_os
    check_dependencies
    install_docker_prompt
    create_directories
    download_binary
    setup_path
    setup_openai_key
    start_qdrant
    print_success
}

main "$@"
