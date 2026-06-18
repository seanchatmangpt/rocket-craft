#!/usr/bin/env bash

# clean-all.sh
# Safely removes 'Binaries/', 'Intermediate/', and 'Saved/' folders across all sub-projects.

set -euo pipefail

# Determine script directory dynamically
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
echo -e "${BOLD}${CYAN}          Rocket Craft Sub-Project Clean            ${RESET}"
echo -e "${BOLD}${CYAN}====================================================${RESET}"
log_info "Workspace root: ${SCRIPT_DIR}"

VERSIONS_DIR="${SCRIPT_DIR}/versions"
if [ ! -d "${VERSIONS_DIR}" ]; then
    log_warn "Versions directory does not exist: ${VERSIONS_DIR}. Nothing to clean."
    exit 0
fi

# Folders to target
TARGETS=("Binaries" "Intermediate" "Saved")
deleted_count=0

for target in "${TARGETS[@]}"; do
    log_info "Searching for '${target}' folders in versions/..."
    
    # We use a temporary file to collect paths safely
    temp_file=$(mktemp)
    find "${VERSIONS_DIR}" -type d -name "${target}" -prune -print0 > "${temp_file}"
    
    while IFS= read -r -d '' dir; do
        if [ -d "${dir}" ]; then
            log_info "Deleting: ${dir}"
            rm -rf "${dir}"
            deleted_count=$((deleted_count + 1))
        fi
    done < "${temp_file}"
    rm -f "${temp_file}"
done

echo -e "${BOLD}${CYAN}----------------------------------------------------${RESET}"
if [ "${deleted_count}" -gt 0 ]; then
    log_success "Cleanup complete. Removed ${deleted_count} target directories."
else
    log_success "Cleanup complete. No target directories were found."
fi
echo -e "${BOLD}${CYAN}====================================================${RESET}"
