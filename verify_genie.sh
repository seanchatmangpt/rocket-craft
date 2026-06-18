#!/usr/bin/env bash

# verify_genie.sh
# Genie 26 E2E Integration and Validation Script
# Compiles unify-rs, runs manufacturing pipeline, evolves spec, and tests local server.

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
echo -e "${BOLD}${CYAN}  Genie 26 System Integration and Validation        ${RESET}"
echo -e "${BOLD}${CYAN}====================================================${RESET}"
log_info "Working directory: ${CWD}"

# Resolve UE4_ROOT dynamically
if [ -z "${UE4_ROOT:-}" ]; then
    CONFIG_FILE="${CWD}/.rocket.json"
    if [ -f "${CONFIG_FILE}" ]; then
        UE4_ROOT_CONFIG=$(python3 -c "import json; d=json.load(open('${CONFIG_FILE}')); print(d.get('ue4_root', ''))" 2>/dev/null || true)
        if [ -n "${UE4_ROOT_CONFIG}" ]; then
            export UE4_ROOT="${UE4_ROOT_CONFIG}"
        fi
    fi
fi

if [ -z "${UE4_ROOT:-}" ]; then
    log_error "UE4_ROOT not set. Please set UE4_ROOT to a valid Unreal Engine 4.27 HTML5 ES3 installation."
    exit 1
fi
log_info "UE4_ROOT resolved to: ${UE4_ROOT}"

# Check arguments: default to non-interactive mode
INTERACTIVE=false
for arg in "$@"; do
    if [ "${arg}" = "--interactive" ] || [ "${arg}" = "-i" ]; then
        INTERACTIVE=true
    fi
done

# 1. Compile the workspace
log_info "[1/6] Compiling unify-rs workspace..."
if ! (cd "${CWD}/unify-rs" && cargo build); then
    log_error "Compilation of unify-rs workspace failed."
    exit 1
fi

UNIFY_BIN="${CWD}/unify-rs/target/debug/unify"
if [ ! -f "${UNIFY_BIN}" ]; then
    log_error "Unify CLI binary not found at ${UNIFY_BIN}"
    exit 1
fi
log_success "CLI binary verified: ${UNIFY_BIN}"

# 2. Cleanup previous outputs
log_info "[2/6] Cleaning up previous outputs..."
rm -f "${CWD}/spec.json" "${CWD}/map.t3d" "${CWD}/deploy.log" "${CWD}/init_intent.txt" "${CWD}/mod_intent.txt"

# 3. Create initial world intent and manufacture
log_info "[3/6] Manufacturing initial world..."
cat << 'EOF' > "${CWD}/init_intent.txt"
create place room_1 name "Control Room" at (0.0, 0.0, 0.0) bounds (100.0, 100.0, 50.0)
create actor bot_1 name "Welder Bot" role RoboticWelder in room_1
create object cnc_1 name "CNC Alpha" class CNC_Machine in room_1
create relationship rel_1 contains from room_1 to bot_1
create rule rule_1 name TempCheck expression "room_1.temp < 30" severity error
EOF

if ! "${UNIFY_BIN}" genie manufacture \
  --intent "${CWD}/init_intent.txt" \
  --out-spec "${CWD}/spec.json" \
  --out-t3d "${CWD}/map.t3d"; then
    log_error "World manufacturing step failed."
    exit 1
fi

# Verify initial artifacts
if [ ! -f "${CWD}/spec.json" ] || [ ! -f "${CWD}/map.t3d" ]; then
    log_error "Initial manufacturing failed to produce spec.json or map.t3d"
    exit 1
fi

log_success "Initial spec.json and map.t3d manufactured successfully!"
if ! grep -q "Begin Map" "${CWD}/map.t3d" || \
   ! grep -q "Place_room_1" "${CWD}/map.t3d" || \
   ! grep -q "Actor_bot_1" "${CWD}/map.t3d" || \
   ! grep -q "Object_cnc_1" "${CWD}/map.t3d"; then
    log_error "T3D layout check failed: missing required elements."
    exit 1
fi
log_success "T3D structural layout conforms to Unreal Engine 4 specifications."

# 4. Evolve the world with updates
log_info "[4/6] Evolving world with incremental modification intent..."
cat << 'EOF' > "${CWD}/mod_intent.txt"
create place room_2 name "Storage Room" at (200.0, 0.0, 0.0) bounds (50.0, 50.0, 30.0)
update actor bot_1 position (20.0, -10.0, 0.0)
create relationship rel_2 connects from room_1 to room_2
EOF

if ! "${UNIFY_BIN}" genie evolve \
  --spec "${CWD}/spec.json" \
  --intent "${CWD}/mod_intent.txt" \
  --out-spec "${CWD}/spec.json" \
  --out-t3d "${CWD}/map.t3d"; then
    log_error "World evolution step failed."
    exit 1
fi

# Verify evolved artifacts
if ! grep -q "Place_room_2" "${CWD}/map.t3d" || \
   ! grep -q "RelativeLocation=(X=20.000000,Y=-10.000000,Z=0.000000)" "${CWD}/map.t3d"; then
    log_error "Evolved T3D checks failed."
    exit 1
fi
log_success "Evolved spec.json and map.t3d verified. State continuity preserved!"

# 5. Deploy the world (generate telemetry log)
log_info "[5/6] Deploying world telemetry logs..."
if ! "${UNIFY_BIN}" genie deploy \
  --spec "${CWD}/spec.json" \
  --log "${CWD}/deploy.log"; then
    log_error "World deployment step failed."
    exit 1
fi

if [ ! -f "${CWD}/deploy.log" ]; then
    log_error "Deployment failed to produce deploy.log"
    exit 1
fi
log_success "Deployment log created successfully."
if ! grep -q "Places: 2" "${CWD}/deploy.log" || ! grep -q "Actors: 1" "${CWD}/deploy.log"; then
    log_error "Deployment log missing places/actors summary."
    exit 1
fi

# Clean up temp intent files
rm -f "${CWD}/init_intent.txt" "${CWD}/mod_intent.txt"

# 6. Launch the interactive simulator server
log_info "[6/6] Launching local Web Operating Center server..."

# Check if port 3000 is in use and kill it to prevent conflicts
if lsof -Pi :3000 -sTCP:LISTEN -t >/dev/null ; then
    log_warn "Port 3000 is in use, cleaning up..."
    pid=$(lsof -t -i:3000)
    kill "${pid}" || kill -9 "${pid}" || true
    sleep 1
fi

node "${CWD}/genie_server.js" &
SERVER_PID=$!

# Trap exit to kill the server when the script ends
trap 'kill "${SERVER_PID}" 2>/dev/null || true' EXIT

log_info "Starting Node server on port 3000 in background (PID: ${SERVER_PID})..."
sleep 2

# Verify server is listening
if ! lsof -Pi :3000 -sTCP:LISTEN -t >/dev/null; then
    log_error "Server failed to start on port 3000!"
    exit 1
fi

log_success "Server is listening on port 3000."

echo -e "${BOLD}${CYAN}----------------------------------------------------${RESET}"
log_success "GENIE 26 SYSTEM VALIDATION SUCCESSFUL!"
log_info "Open your browser and navigate to: http://localhost:3000"
echo -e "${BOLD}${CYAN}----------------------------------------------------${RESET}"

if [ "${INTERACTIVE}" = "true" ]; then
    log_info "Interactive mode active. Press Ctrl+C to stop the server and exit."
    wait
else
    log_info "Non-interactive default: validation completed successfully. Shutting down server."
    exit 0
fi
