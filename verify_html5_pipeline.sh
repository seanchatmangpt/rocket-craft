#!/usr/bin/env bash

# verify_html5_pipeline.sh
# Automated end-to-end HTML5 pipeline and E2E Playwright verification script.

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
echo -e "${BOLD}${CYAN}      HTML5 Pipeline and Playwright E2E Verification${RESET}"
echo -e "${BOLD}${CYAN}====================================================${RESET}"
log_info "Working directory: ${CWD}"

# 1. Clean previous build outputs
log_info "[1/8] Cleaning previous build outputs..."
rm -f "${CWD}/spec.json" "${CWD}/map.t3d" "${CWD}/deploy.log" "${CWD}/init_intent.txt" "${CWD}/mod_intent.txt"
rm -rf "${CWD}/pwa-staff/manufactured"
rm -f "${CWD}/pwa-staff/test-results/tps-dflss-receipt.json" "${CWD}/pwa-staff/test-results/tps-dflss-diff.png"

# 2. Resolve UE4_ROOT dynamically
log_info "[2/8] Resolving UE4_ROOT..."
if [ -z "${UE4_ROOT:-}" ]; then
    # Try reading from config file
    CONFIG_FILE="${CWD}/.rocket.json"
    if [ -f "${CONFIG_FILE}" ]; then
        # Use simple python script to read json safely without dependencies
        UE4_ROOT_CONFIG=$(python3 -c "import json, sys; d=json.load(open('${CONFIG_FILE}')); print(d.get('ue4_root', ''))" 2>/dev/null || true)
        if [ -n "${UE4_ROOT_CONFIG}" ]; then
            export UE4_ROOT="${UE4_ROOT_CONFIG}"
        fi
    fi
fi

# Fail if not resolved
if [ -z "${UE4_ROOT:-}" ]; then
    log_error "UE4_ROOT not set. Please set UE4_ROOT to a valid Unreal Engine 4.27 HTML5 ES3 installation."
    exit 1
fi
log_info "UE4_ROOT resolved to: ${UE4_ROOT}"

# 3. Compile the backend (unify-rs)
log_info "[3/8] Compiling backend (unify-rs)..."
if ! (cd "${CWD}/unify-rs" && cargo build); then
    log_error "Failed to compile unify-rs backend."
    exit 1
fi

UNIFY_BIN="${CWD}/unify-rs/target/debug/unify"
if [ ! -f "${UNIFY_BIN}" ]; then
    log_error "Unify CLI binary not found at ${UNIFY_BIN}"
    exit 1
fi
log_success "Backend compiled and verified successfully."

# 4. Trigger world manufacture, evolution, and simulated UE4 packaging pipeline
log_info "[4/8] Triggering world manufacture and evolution..."

# Create initial world intent
cat << 'EOF' > "${CWD}/init_intent.txt"
create place zone_1 name "Primitive Foundry" at (0.0, 0.0, 0.0) bounds (150.0, 150.0, 50.0)
create place zone_2 name "Part Runner Wall" at (400.0, 0.0, 0.0) bounds (150.0, 150.0, 50.0)
create place zone_3 name "Assembly Gantry" at (800.0, 0.0, 0.0) bounds (150.0, 150.0, 50.0)
create place zone_4 name "Fit + Collision Bay" at (1200.0, 0.0, 0.0) bounds (150.0, 150.0, 50.0)
create place zone_5 name "Physics Proving Ground" at (1600.0, 0.0, 0.0) bounds (150.0, 150.0, 50.0)
create place zone_6 name "Final Reveal Platform" at (2000.0, 0.0, 0.0) bounds (150.0, 150.0, 50.0)
create relationship rel_1_2 connects from zone_1 to zone_2
create relationship rel_2_3 connects from zone_2 to zone_3
create relationship rel_3_4 connects from zone_3 to zone_4
create relationship rel_4_5 connects from zone_4 to zone_5
create relationship rel_5_6 connects from zone_5 to zone_6
EOF

# Manufacture
if ! "${UNIFY_BIN}" genie manufacture \
  --intent "${CWD}/init_intent.txt" \
  --out-spec "${CWD}/spec.json" \
  --out-t3d "${CWD}/map.t3d"; then
    log_error "genie manufacture command failed."
    exit 1
fi

# Create evolution intent
cat << 'EOF' > "${CWD}/mod_intent.txt"
create actor bot_1 name "Welder Bot" role RoboticWelder in zone_1
create place zone_extra name "Extra Lab" at (2400.0, 0.0, 0.0) bounds (100.0, 100.0, 50.0)
create relationship rel_6_extra connects from zone_6 to zone_extra
EOF

# Evolve
if ! "${UNIFY_BIN}" genie evolve \
  --spec "${CWD}/spec.json" \
  --intent "${CWD}/mod_intent.txt" \
  --out-spec "${CWD}/spec.json" \
  --out-t3d "${CWD}/map.t3d"; then
    log_error "genie evolve command failed."
    exit 1
fi

# Deploy (triggers UE4 simulation build/package pipeline and copies HTML5 package to served directory)
log_info "Deploying world layout and running simulated UE4 pipeline..."
if ! "${UNIFY_BIN}" genie deploy \
  --spec "${CWD}/spec.json" \
  --log "${CWD}/deploy.log"; then
    log_error "genie deploy command failed."
    exit 1
fi

# Clean up temp intent files
rm -f "${CWD}/init_intent.txt" "${CWD}/mod_intent.txt"
log_success "World artifacts manufactured, evolved, and deployed."

# 5. Start the local web server on port 3000
log_info "[5/8] Starting local web server on port 3000..."
if lsof -Pi :3000 -sTCP:LISTEN -t >/dev/null ; then
    log_warn "Port 3000 in use, stopping existing process..."
    pid=$(lsof -t -i:3000)
    kill "${pid}" || kill -9 "${pid}" || true
    sleep 1
fi

node "${CWD}/genie_server.js" &
SERVER_PID=$!

# Register exit trap to clean up server PID immediately if anything fails or exits
trap 'log_info "[8/8] Shutting down local web server (PID: ${SERVER_PID})..."; kill "${SERVER_PID}" 2>/dev/null || true' EXIT

# Wait for server to initialize
sleep 2

# Verify server is listening
if ! lsof -Pi :3000 -sTCP:LISTEN -t >/dev/null; then
    log_error "Server failed to start on port 3000!"
    exit 1
fi
log_success "Server is listening on port 3000."

# 6. Run the Playwright test
log_info "[6/8] Running Playwright E2E test..."
PLAYWRIGHT_EXIT=0
# Run playwright in subshell to isolate directory navigation
if ! (cd "${CWD}/pwa-staff" && npx playwright test tests-e2e/tps-dflss.spec.ts); then
    log_warn "Playwright E2E test suite reported errors or failures."
    PLAYWRIGHT_EXIT=1
fi

# 7. Validate that tps-dflss-receipt.json is generated successfully with verdict PASS and all fields present
log_info "[7/8] Validating generated receipt..."
VALIDATION_EXIT=0
if ! node -e '
const fs = require("fs");
const path = require("path");
const receiptPath = path.join("pwa-staff", "test-results", "tps-dflss-receipt.json");

if (!fs.existsSync(receiptPath)) {
  console.error("Error: Receipt file tps-dflss-receipt.json was not generated!");
  process.exit(1);
}

const receipt = JSON.parse(fs.readFileSync(receiptPath, "utf8"));
const requiredFields = [
  "prompt", "contractHash", "buildLog", "packagePath",
  "browserUrl", "screenshots", "consoleLogs", "inputTrace",
  "visualDelta", "verdict"
];

for (const field of requiredFields) {
  if (receipt[field] === undefined) {
    console.error(`Error: Missing required field "${field}" in receipt!`);
    process.exit(1);
  }
}

if (receipt.verdict !== "PASS") {
  console.error(`Error: Expected verdict "PASS", got "${receipt.verdict}"`);
  process.exit(1);
}

if (!receipt.screenshots || receipt.screenshots.before === undefined || receipt.screenshots.after === undefined) {
  console.error("Error: Screenshots must contain before and after base64 strings");
  process.exit(1);
}

console.log("Success: Receipt validation passed successfully!");
process.exit(0);
'; then
    log_error "Affidavit receipt validation failed."
    VALIDATION_EXIT=1
fi

echo -e "${BOLD}${CYAN}----------------------------------------------------${RESET}"
if [ ${PLAYWRIGHT_EXIT} -eq 0 ] && [ ${VALIDATION_EXIT} -eq 0 ]; then
    log_success "E2E HTML5 PIPELINE VERIFICATION SUCCESSFUL!"
    echo -e "${BOLD}${CYAN}----------------------------------------------------${RESET}"
    exit 0
else
    log_error "E2E HTML5 PIPELINE VERIFICATION FAILED!"
    log_error "Playwright Exit Code: ${PLAYWRIGHT_EXIT}"
    log_error "Validation Exit Code: ${VALIDATION_EXIT}"
    echo -e "${BOLD}${RED}----------------------------------------------------${RESET}"
    exit 1
fi
