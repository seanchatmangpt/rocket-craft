#!/usr/bin/env bash
# Stage 6 — Post-build serve + Playwright proof
# Prerequisites: Stage 5 complete (StagedBuilds/HTML5/ populated by RunUAT)
#
# What this script does:
#   1. Copies real HTML5 package from StagedBuilds into pwa-staff/manufactured/
#   2. Verifies the .wasm is real (size > 10 MB, magic bytes 0061736d)
#   3. Installs pwa-staff node_modules if missing (npm ci)
#   4. Installs Playwright browser binaries if missing
#   5. Starts genie_server.js on port 3000 (Playwright config reuses it)
#   6. Runs: cd pwa-staff && npx playwright test tests-e2e/tps-dflss.spec.ts
#   7. Prints receipt path and verdict

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG="$HOME/ue4-build.log"

# ── Source .rocket.json for UE4_ROOT ────────────────────────────────────────
ROCKET_JSON="$SCRIPT_DIR/.rocket.json"
if [[ ! -f "$ROCKET_JSON" ]]; then
    echo "BLOCKED: $ROCKET_JSON not found — cannot determine ue4_root"
    exit 1
fi

if command -v jq &>/dev/null; then
    UE4_ROOT="$(jq -r '.ue4_root // empty' "$ROCKET_JSON")"
elif command -v python3 &>/dev/null; then
    UE4_ROOT="$(python3 -c "import json,sys; d=json.load(open('$ROCKET_JSON')); print(d.get('ue4_root',''))")"
else
    echo "BLOCKED: neither jq nor python3 available to parse .rocket.json"
    exit 1
fi

if [[ -z "$UE4_ROOT" ]]; then
    echo "BLOCKED: ue4_root not set in $ROCKET_JSON"
    exit 1
fi

log() {
    local ts
    ts="$(date '+%Y-%m-%d %H:%M:%S')"
    echo "[$ts] $*" | tee -a "$LOG"
}

log "========================================"
log "STAGE 6 — Post-build serve + Playwright proof"
log "========================================"

# ── Paths ────────────────────────────────────────────────────────────────────
STAGED_BUILDS="$SCRIPT_DIR/versions/4.27.0/Saved/StagedBuilds/HTML5"
MANUFACTURED="$SCRIPT_DIR/pwa-staff/manufactured"
PWA_DIR="$SCRIPT_DIR/pwa-staff"
RECEIPT_DIR="$PWA_DIR/test-results"
RECEIPT_PATH="$RECEIPT_DIR/tps-dflss-receipt.json"
GENIE_SERVER="$SCRIPT_DIR/genie_server.js"

# ── Step 1: Copy real package into pwa-staff/manufactured/ ──────────────────
log "[1/6] Copying HTML5 package from StagedBuilds into manufactured/..."

if [[ ! -d "$STAGED_BUILDS" ]]; then
    echo "BLOCKED: Stage 5 output directory not found: $STAGED_BUILDS"
    echo "Run package-brm-html5.sh first (Stage 5)."
    exit 1
fi

# Count qualifying files
STAGE5_FILES=$(find "$STAGED_BUILDS" -maxdepth 1 \
    \( -name "*.html" -o -name "*.js" -o -name "*.wasm" -o -name "*.data" \) \
    -type f 2>/dev/null | wc -l | tr -d ' ')

if [[ "$STAGE5_FILES" -eq 0 ]]; then
    echo "BLOCKED: No HTML5 package files found in $STAGED_BUILDS"
    echo "Expected *.html *.js *.wasm *.data — run Stage 5 first."
    exit 1
fi

mkdir -p "$MANUFACTURED"
# Copy only the HTML5 game files (not subdirectories, not receipt.json etc.)
find "$STAGED_BUILDS" -maxdepth 1 \
    \( -name "*.html" -o -name "*.js" -o -name "*.wasm" -o -name "*.data" \) \
    -type f \
    -exec cp -v {} "$MANUFACTURED/" \;

log "Copied $STAGE5_FILES file(s) into $MANUFACTURED"

# ── Step 2: Verify .wasm (size > 10 MB, magic bytes 0061736d) ───────────────
log "[2/6] Verifying WASM integrity..."

WASM_FILE=$(find "$MANUFACTURED" -name "*.wasm" -type f 2>/dev/null | head -n 1)
if [[ -z "$WASM_FILE" ]]; then
    echo "BLOCKED: No .wasm file found in $MANUFACTURED after copy"
    exit 1
fi

# Portable file size
WASM_SIZE=$(stat -f%z "$WASM_FILE" 2>/dev/null || stat -c%s "$WASM_FILE" 2>/dev/null)
MIN_WASM=$((10 * 1024 * 1024))  # 10 MB

if [[ "$WASM_SIZE" -le "$MIN_WASM" ]]; then
    WASM_MB=$(( WASM_SIZE / 1024 / 1024 ))
    echo "BLOCKED: WASM file too small: $WASM_FILE (${WASM_MB} MB — expected >10 MB)"
    echo "The file at $STAGED_BUILDS may be a stub. Re-run Stage 5 with a real UE4 build."
    exit 1
fi

MAGIC=$(xxd -p -l 4 "$WASM_FILE" 2>/dev/null \
    || od -An -tx1 -N4 "$WASM_FILE" 2>/dev/null | tr -d ' \n')
MAGIC_NORM=$(echo "$MAGIC" | tr '[:upper:]' '[:lower:]' | tr -d ' \n')
EXPECTED_MAGIC="0061736d"

if [[ "$MAGIC_NORM" != "$EXPECTED_MAGIC" ]]; then
    echo "BLOCKED: WASM magic bytes invalid — got '$MAGIC_NORM', expected '$EXPECTED_MAGIC'"
    echo "File: $WASM_FILE"
    exit 1
fi

WASM_MB=$(( WASM_SIZE / 1024 / 1024 ))
log "WASM verified: $WASM_FILE (${WASM_MB} MB, magic 0061736d OK)"

# ── Step 3: Install node_modules if missing ──────────────────────────────────
log "[3/6] Checking pwa-staff node_modules..."

NODE_MODULES="$PWA_DIR/node_modules"
if [[ ! -d "$NODE_MODULES" ]] || [[ -z "$(ls -A "$NODE_MODULES" 2>/dev/null)" ]]; then
    log "node_modules missing or empty — running npm ci in $PWA_DIR"
    (cd "$PWA_DIR" && npm ci)
    log "npm ci complete"
else
    log "node_modules present — skipping npm ci"
fi

# ── Step 4: Install Playwright browser binaries if missing ───────────────────
log "[4/6] Checking Playwright browser binaries..."

CHROMIUM_DIR=$(find "$NODE_MODULES/.cache/ms-playwright" -name "chromium-*" -maxdepth 1 -type d 2>/dev/null | head -n 1 || true)
# Also check the canonical ~/.cache/ms-playwright location
CHROMIUM_HOME=$(find "$HOME/Library/Caches/ms-playwright" -name "chromium-*" -maxdepth 1 -type d 2>/dev/null | head -n 1 || true)

if [[ -z "$CHROMIUM_DIR" ]] && [[ -z "$CHROMIUM_HOME" ]]; then
    log "Playwright Chromium not found — running npx playwright install chromium"
    (cd "$PWA_DIR" && npx playwright install chromium)
    log "Playwright browser install complete"
else
    log "Playwright Chromium present — skipping install"
fi

# ── Step 5: Start genie_server.js on port 3000 ──────────────────────────────
log "[5/6] Starting genie_server.js on port 3000..."

SERVER_PID=""

# Kill any existing occupant of port 3000
if lsof -Pi :3000 -sTCP:LISTEN -t >/dev/null 2>&1; then
    log "Port 3000 already in use — stopping existing process"
    EXISTING_PID=$(lsof -t -i:3000 2>/dev/null || true)
    if [[ -n "$EXISTING_PID" ]]; then
        kill "$EXISTING_PID" 2>/dev/null || true
        sleep 1
    fi
fi

node "$GENIE_SERVER" &
SERVER_PID=$!
log "genie_server.js started (PID $SERVER_PID)"

# Trap ensures the server is stopped on exit regardless of test outcome
cleanup() {
    if [[ -n "$SERVER_PID" ]]; then
        log "Stopping genie_server.js (PID $SERVER_PID)..."
        kill "$SERVER_PID" 2>/dev/null || true
    fi
}
trap cleanup EXIT

# Wait for server to bind
sleep 2

if ! lsof -Pi :3000 -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo "BLOCKED: genie_server.js failed to bind to port 3000"
    exit 1
fi
log "Server listening on port 3000"

# ── Step 6: Run Playwright test ──────────────────────────────────────────────
log "[6/6] Running Playwright E2E test (tps-dflss.spec.ts)..."

PLAYWRIGHT_EXIT=0
(cd "$PWA_DIR" && npx playwright test tests-e2e/tps-dflss.spec.ts) || PLAYWRIGHT_EXIT=$?

# ── Print verdict ─────────────────────────────────────────────────────────────
echo ""
echo "================================================"
if [[ -f "$RECEIPT_PATH" ]]; then
    VERDICT=$(node -e "
const r = JSON.parse(require('fs').readFileSync('$RECEIPT_PATH', 'utf8'));
process.stdout.write(r.verdict || 'UNKNOWN');
" 2>/dev/null || echo "UNKNOWN")
    echo "Receipt: $RECEIPT_PATH"
    echo "Verdict: $VERDICT"
    if [[ "$VERDICT" == "PASS" ]] && [[ "$PLAYWRIGHT_EXIT" -eq 0 ]]; then
        echo ""
        echo "STAGE 6 COMPLETE — PASS"
        echo "================================================"
        log "STAGE 6 COMPLETE — PASS"
        exit 0
    else
        echo ""
        echo "STAGE 6 COMPLETE — FAIL (Playwright exit: $PLAYWRIGHT_EXIT, verdict: $VERDICT)"
        echo "================================================"
        log "STAGE 6 COMPLETE — FAIL"
        exit 1
    fi
else
    echo "Receipt not generated: $RECEIPT_PATH"
    echo ""
    if [[ "$PLAYWRIGHT_EXIT" -ne 0 ]]; then
        echo "STAGE 6 FAILED — Playwright exit code $PLAYWRIGHT_EXIT, no receipt written"
        echo "================================================"
        log "STAGE 6 FAILED — no receipt, playwright exit $PLAYWRIGHT_EXIT"
        exit 1
    else
        echo "STAGE 6 COMPLETE — Playwright passed but no receipt written"
        echo "================================================"
        log "STAGE 6 COMPLETE — no receipt"
        exit 0
    fi
fi
