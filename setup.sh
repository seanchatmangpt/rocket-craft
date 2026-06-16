#!/bin/bash
set -e

# Rocket Craft Setup Proxy Script
# This script ensures Rust is installed and then proxies to './rocket setup'

# Colors for rich formatting
BOLD='\033[1m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BOLD}${CYAN}================================================${NC}"
echo -e "${BOLD}${CYAN}      Rocket Craft Project Bootstrapper         ${NC}"
echo -e "${BOLD}${CYAN}================================================${NC}"

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# 1. Check for Rust/Cargo
if ! command_exists cargo; then
    echo -e "${YELLOW}Rust/Cargo not found. Attempting to install...${NC}"
    
    # Try to install rustup
    if command_exists curl; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        
        # Source cargo env for the current session
        source "$HOME/.cargo/env"
        
        if command_exists cargo; then
            echo -e "${GREEN}Rust installed successfully!${NC}"
        else
            echo -e "${RED}Rust installation failed. Please install it manually from https://rustup.rs/${NC}"
            exit 1
        fi
    else
        echo -e "${RED}Error: 'curl' is required to install Rust automatically.${NC}"
        echo -e "${YELLOW}Please install Rust manually from https://rustup.rs/${NC}"
        exit 1
    fi
else
    echo -e "${GREEN}✓ Rust/Cargo detected.${NC}"
fi

# 2. Proxy to ./rocket setup
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
if [ -f "$SCRIPT_DIR/rocket" ]; then
    echo -e "${CYAN}Proxying to ./rocket setup...${NC}"
    echo ""
    chmod +x "$SCRIPT_DIR/rocket"
    exec "$SCRIPT_DIR/rocket" setup
else
    echo -e "${RED}Error: 'rocket' script not found in $SCRIPT_DIR${NC}"
    exit 1
fi
