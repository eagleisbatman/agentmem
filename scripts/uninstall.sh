#!/usr/bin/env bash
#
# AgentMem Uninstaller
# Usage: curl -sSL https://agentmem.dev/uninstall.sh | bash
#

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

CHECK="${GREEN}✓${NC}"
CROSS="${RED}✗${NC}"
ARROW="${BLUE}→${NC}"

INSTALL_DIR="${HOME}/.local/bin"
CONFIG_DIR="${HOME}/.agentmem"
BINARY_PATH="${INSTALL_DIR}/am"

echo ""
echo -e "${YELLOW}AgentMem Uninstaller${NC}"
echo ""

# Confirm uninstall
echo "This will remove:"
echo "  - Binary: ${BINARY_PATH}"
echo "  - Config: ${CONFIG_DIR}"
echo "  - Qdrant container: agentmem-qdrant (optional)"
echo ""

read -p "Are you sure you want to uninstall AgentMem? [y/N] " -n 1 -r
echo ""

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Cancelled."
    exit 0
fi

# Remove binary
echo ""
echo -e "${ARROW} Removing binary..."
if [ -f "$BINARY_PATH" ]; then
    rm -f "$BINARY_PATH"
    echo -e "  ${CHECK} Removed ${BINARY_PATH}"
else
    echo -e "  ${YELLOW}!${NC} Binary not found"
fi

# Remove config directory
echo ""
echo -e "${ARROW} Removing config directory..."
read -p "Remove global config (~/.agentmem)? This includes credentials. [y/N] " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
    if [ -d "$CONFIG_DIR" ]; then
        rm -rf "$CONFIG_DIR"
        echo -e "  ${CHECK} Removed ${CONFIG_DIR}"
    else
        echo -e "  ${YELLOW}!${NC} Config directory not found"
    fi
else
    echo -e "  ${YELLOW}!${NC} Keeping ${CONFIG_DIR}"
fi

# Remove Qdrant container
echo ""
echo -e "${ARROW} Qdrant container..."

if command -v docker &> /dev/null; then
    if docker ps -a --format '{{.Names}}' | grep -q "^agentmem-qdrant$"; then
        read -p "Remove Qdrant container and data? [y/N] " -n 1 -r
        echo ""

        if [[ $REPLY =~ ^[Yy]$ ]]; then
            docker stop agentmem-qdrant > /dev/null 2>&1 || true
            docker rm agentmem-qdrant > /dev/null 2>&1 || true
            docker volume rm agentmem-qdrant-data > /dev/null 2>&1 || true
            echo -e "  ${CHECK} Removed Qdrant container and data"
        else
            echo -e "  ${YELLOW}!${NC} Keeping Qdrant container"
        fi
    else
        echo -e "  ${YELLOW}!${NC} Qdrant container not found"
    fi
else
    echo -e "  ${YELLOW}!${NC} Docker not installed"
fi

# Note about project directories
echo ""
echo -e "${YELLOW}Note:${NC} Project-specific .agentmem/ directories are not removed."
echo "      Delete them manually from each project if needed."
echo ""

echo -e "${GREEN}AgentMem uninstalled successfully.${NC}"
echo ""
