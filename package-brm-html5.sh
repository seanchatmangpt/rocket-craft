#!/usr/bin/env bash
# Stage 5 — Package Brm for HTML5 via RunUAT
# Requires Stage 4 complete (UE4Editor built)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROCKET_JSON="$SCRIPT_DIR/.rocket.json"
LOG="$HOME/ue4-build.log"
UPROJECT="/Users/sac/rocket-craft/versions/4.27.0/Brm.uproject"
ARCHIVE_DIR="/Users/sac/rocket-craft/pwa-staff/manufactured"

log() {
    local ts
    ts="$(date '+%Y-%m-%d %H:%M:%S')"
    echo "[$ts] $*" | tee -a "$LOG"
}

fail() {
    local reason="$1"
    local logfile="${2:-$LOG}"
    log "ERROR: $reason"
    echo ""
    echo "BLOCKED: $reason"
    echo ""
    echo "--- last 50 lines of RunUAT log ($logfile) ---"
    tail -n 50 "$logfile"
    exit 1
}

log "========================================"
log "STAGE 5 — Package Brm HTML5"
log "========================================"

# Source .rocket.json to get ue4_root
if [[ ! -f "$ROCKET_JSON" ]]; then
    echo "BLOCKED: $ROCKET_JSON not found — cannot determine ue4_root"
    exit 1
fi

# Parse ue4_root from .rocket.json (requires python3 or jq)
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

log "UE4_ROOT: $UE4_ROOT"
export UE4_ROOT

# Gate: Stage 4 must be complete
EDITOR_BIN="$UE4_ROOT/Engine/Binaries/Mac/UE4Editor"
log "Checking UE4Editor exists (Stage 4 gate)..."
if [[ ! -f "$EDITOR_BIN" ]]; then
    echo ""
    echo "BLOCKED: Stage 4 not complete — $EDITOR_BIN not found"
    echo "Run build-ue4editor.sh first"
    exit 1
fi
log "Gate passed: $EDITOR_BIN found"

# Prepare archive directory
mkdir -p "$ARCHIVE_DIR"

UAT_SH="$UE4_ROOT/Engine/Build/BatchFiles/RunUAT.sh"
if [[ ! -f "$UAT_SH" ]]; then
    echo "BLOCKED: RunUAT.sh not found at $UAT_SH"
    exit 1
fi

UAT_LOG="$HOME/ue4-uat-brm-html5.log"
log "Running RunUAT BuildCookRun for HTML5..."
log "UAT log: $UAT_LOG"

if ! arch -x86_64 bash "$UAT_SH" BuildCookRun \
    -project="$UPROJECT" \
    -platform=HTML5 \
    -clientconfig=Shipping \
    -cook \
    -stage \
    -package \
    -archivedirectory="$ARCHIVE_DIR" \
    -noP4 \
    >> "$UAT_LOG" 2>&1; then
    fail "RunUAT BuildCookRun (HTML5) FAILED" "$UAT_LOG"
fi

log "RunUAT finished — verifying WASM output"

# Find .wasm file in output
WASM_FILE=$(find "$ARCHIVE_DIR" -name "*.wasm" -type f 2>/dev/null | head -n 1)

if [[ -z "$WASM_FILE" ]]; then
    fail "No .wasm file found in $ARCHIVE_DIR" "$UAT_LOG"
fi

WASM_SIZE=$(stat -f%z "$WASM_FILE" 2>/dev/null || stat -c%s "$WASM_FILE" 2>/dev/null)
MIN_WASM=$((10 * 1024 * 1024))  # 10 MB

if [[ "$WASM_SIZE" -le "$MIN_WASM" ]]; then
    fail "WASM file too small: $WASM_FILE (${WASM_SIZE} bytes, expected >10 MB)" "$UAT_LOG"
fi

# Check WebAssembly magic bytes: 0x00 0x61 0x73 0x6d
MAGIC=$(xxd -p -l 4 "$WASM_FILE" 2>/dev/null || od -An -tx1 -N4 "$WASM_FILE" 2>/dev/null | tr -d ' \n')
EXPECTED="0061736d"

# Normalise to lowercase, strip spaces
MAGIC_NORM=$(echo "$MAGIC" | tr '[:upper:]' '[:lower:]' | tr -d ' \n')

if [[ "$MAGIC_NORM" != "$EXPECTED" ]]; then
    fail "WASM magic bytes invalid: got '$MAGIC_NORM', expected '$EXPECTED'" "$UAT_LOG"
fi

WASM_MB=$(( WASM_SIZE / 1024 / 1024 ))
log "WASM verified: $WASM_FILE (${WASM_MB} MB, magic 0061736d OK)"

echo ""
echo "Package path: $WASM_FILE"
echo "WASM size:    ${WASM_MB} MB"
echo "Magic bytes:  0061736d (valid WebAssembly)"
echo ""
echo "STAGE 5 COMPLETE"
log "STAGE 5 COMPLETE"
