#!/usr/bin/env bash

# verify_genie3.sh
# Genie 3 Verification Script
# Builds the genie3-rs crate and runs integration tests to verify the scenario.

set -euo pipefail

# Dynamic script directory resolution
CWD="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

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
echo -e "${BOLD}${CYAN}  Genie 3 World Model Verification Scenario         ${RESET}"
echo -e "${BOLD}${CYAN}====================================================${RESET}"
log_info "Working directory: ${CWD}"

# 1. Build the crate
log_info "[1/3] Building genie3-rs crate..."
if ! (cd "${CWD}/genie3-rs" && cargo build); then
    log_error "Compilation of genie3-rs failed."
    exit 1
fi

# 2. Describe the verification scenario
echo ""
log_info "[2/3] Verification scenario details:"
echo -e "  - ${BOLD}Create Initial State:${RESET}"
echo "    * Place: room_1 (Control Room, hard containment enabled)"
echo "    * Actor: bot_1 (Welder Bot, starting at origin)"
echo "    * Object: cnc_1 (CNC Machine at (10, 10, 0))"
echo -e "  - ${BOLD}Apply Movement Action:${RESET}"
echo "    * Move bot_1 by (+5, +5, 0)"
echo "    * Assert state transition and placement updates"
echo -e "  - ${BOLD}Apply Promptable Events:${RESET}"
echo "    * Event A: Change weather to Stormy, time of day to 20:00 (evening)"
echo "    * Event B: Spawn new barrier object 'barrier_1' at (5, 10, 0)"
echo -e "  - ${BOLD}Validate Consistency & Safety Constraints:${RESET}"
echo "    * Assert that bot_1 cannot move into the spawned barrier (collision avoidance)"
echo "    * Assert that bot_1 cannot teleport (speed limits)"
echo "    * Assert that bot_1 cannot exit room_1 (hard containment)"
echo "    * Validate referential coherence across all entities"
echo ""

# 3. Run the scenario tests
log_info "[3/3] Running integration test suite..."
if (cd "${CWD}/genie3-rs" && cargo test --test integration_tests -- --nocapture); then
    echo ""
    echo -e "${BOLD}${CYAN}====================================================${RESET}"
    log_success "Genie 3 World Model scenario executed successfully!"
    log_info "All state transitions and consistency assertions passed."
    echo -e "${BOLD}${CYAN}====================================================${RESET}"
    exit 0
else
    echo ""
    echo -e "${BOLD}${RED}====================================================${RESET}"
    log_error "Genie 3 World Model verification failed."
    log_error "Some test assertions or state transitions did not pass."
    echo -e "${BOLD}${RED}====================================================${RESET}"
    exit 1
fi
