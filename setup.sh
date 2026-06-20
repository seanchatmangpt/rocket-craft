#!/usr/bin/env bash

# Rocket Craft Setup Proxy Script
# This script ensures Rust is installed and then proxies to './rocket setup'

set -euo pipefail

# Dynamic script directory resolution
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Color support check
if [ -t 1 ] && [ "${NO_COLOR:-}" = "" ]; then
    BOLD="\033[1m"
    RED="\033[31m"
    GREEN="\033[32m"
    YELLOW="\033[33m"
    BLUE="\033[34m"
    CYAN="\033[36m"
    RESET="\033[0m"
else
    BOLD=""
    RED=""
    GREEN=""
    YELLOW=""
    BLUE=""
    CYAN=""
    RESET=""
fi

log_info() { echo -e "${BLUE}${BOLD}[INFO]${RESET} $*"; }
log_success() { echo -e "${GREEN}${BOLD}[SUCCESS]${RESET} $*"; }
log_warn() { echo -e "${YELLOW}${BOLD}[WARN]${RESET} $*"; }
log_error() { echo -e "${RED}${BOLD}[ERROR]${RESET} $*" >&2; }

echo -e "${BOLD}${CYAN}====================================================${RESET}"
echo -e "${BOLD}${CYAN}        Rocket Craft Project Bootstrapper           ${RESET}"
echo -e "${BOLD}${CYAN}====================================================${RESET}"
log_info "Workspace root: ${SCRIPT_DIR}"

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# 1. Check for Rust/Cargo
if ! command_exists cargo; then
    log_warn "Rust/Cargo not found. Attempting to install automatically..."
    
    if command_exists curl; then
        # Install rustup in non-interactive mode
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        
        # Source cargo env for the current session if it exists
        CARGO_ENV="$HOME/.cargo/env"
        if [ -f "${CARGO_ENV}" ]; then
            # Sourcing inside subshell/script won't persist to user's interactive shell,
            # but it is required for the rest of this script's execution.
            # shellcheck disable=SC1090
            source "${CARGO_ENV}"
        else
            log_error "Rust environment file not found at ${CARGO_ENV} after installation."
            exit 1
        fi
        
        if command_exists cargo; then
            log_success "Rust installed successfully!"
        else
            log_error "Rust installation succeeded but 'cargo' is still not found in PATH."
            log_error "Please add $HOME/.cargo/bin to your PATH or restart your terminal."
            exit 1
        fi
    else
        log_error "'curl' is required to install Rust automatically."
        log_warn "Please install Rust manually from https://rustup.rs/ and re-run this script."
        exit 1
    fi
else
    log_success "Rust/Cargo detected: $(cargo --version)"
fi

# 2. Proxy to `./rocket setup`
ROCKET_SCRIPT="${SCRIPT_DIR}/rocket"
if [ -f "${ROCKET_SCRIPT}" ]; then
    log_info "Proxying to ./rocket setup..."
    echo -e "${BOLD}${CYAN}----------------------------------------------------${RESET}"
    chmod +x "${ROCKET_SCRIPT}"
    
    # We execute rocket setup using exec to replace the current shell process
    exec "${ROCKET_SCRIPT}" setup
else
    log_error "'rocket' management script not found in ${SCRIPT_DIR}"
    exit 1
fi
