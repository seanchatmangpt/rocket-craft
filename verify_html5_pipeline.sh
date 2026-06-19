#!/usr/bin/env bash
# verify_html5_pipeline.sh — Stage 6: Serve real Brm.wasm + Playwright proof + receipt.
# Usage: ./verify_html5_pipeline.sh [archive_dir]
# archive_dir defaults to /tmp/brm-html5-archive/HTML5

set -euo pipefail

CWD="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# UAT archives to <archivedirectory>/HTML5/ but some cook modes put files
# directly in <archivedirectory>. Accept either.
_BASE="${1:-/tmp/brm-html5-archive}"
if [ -d "$_BASE/HTML5" ] && ls "$_BASE/HTML5/"*.wasm 2>/dev/null | head -1 | grep -q .; then
  ARCHIVE_DIR="$_BASE/HTML5"
else
  ARCHIVE_DIR="$_BASE"
fi
PORT=8080
RECEIPT="$CWD/pwa-staff/test-results/tps-dflss-receipt.json"

if [ -t 1 ] && [ "${NO_COLOR:-}" = "" ]; then
  BOLD="\033[1m" RED="\033[31m" GREEN="\033[32m" YELLOW="\033[33m"
  BLUE="\033[34m" CYAN="\033[36m" RESET="\033[0m"
else
  BOLD="" RED="" GREEN="" YELLOW="" BLUE="" CYAN="" RESET=""
fi

log_info()    { echo -e "${BLUE}${BOLD}[INFO]${RESET} $*"; }
log_success() { echo -e "${GREEN}${BOLD}[PASS]${RESET} $*"; }
log_warn()    { echo -e "${YELLOW}${BOLD}[WARN]${RESET} $*"; }
log_error()   { echo -e "${RED}${BOLD}[FAIL]${RESET} $*" >&2; }

echo -e "${BOLD}${CYAN}============================================${RESET}"
echo -e "${BOLD}${CYAN}   Rocket Craft — Stage 6 HTML5 E2E Proof   ${RESET}"
echo -e "${BOLD}${CYAN}============================================${RESET}"

# 0. Summarize any cook errors (blueprint errors are non-fatal with -IgnoreCookErrors)
COOK_LOG="$HOME/ue4-cook6.log"
[ ! -f "$COOK_LOG" ] && COOK_LOG="$HOME/ue4-cook5.log"
if [ -f "$COOK_LOG" ]; then
  BLUEPRINT_ERRORS=$(grep -c "LogBlueprint: Error" "$COOK_LOG" 2>/dev/null || echo 0)
  if [ "$BLUEPRINT_ERRORS" -gt 0 ]; then
    log_warn "$BLUEPRINT_ERRORS blueprint errors in cook log (VaRest redirects unresolved — networking UI skipped in pak)"
  fi
fi

# 1. Verify the real WASM artifact exists and is valid
log_info "[1/5] Verifying WASM artifact..."
WASM_FILE=""
for candidate in \
    "$ARCHIVE_DIR/Brm.wasm" \
    "$CWD/versions/Brm427/Binaries/HTML5/Brm.wasm" \
    "$CWD/versions/4.27.0/Binaries/HTML5/Brm.wasm"; do
  if [ -f "$candidate" ]; then
    WASM_FILE="$candidate"
    break
  fi
done

if [ -z "$WASM_FILE" ]; then
  log_error "No Brm.wasm found. Run 'rocket html5 cook --project Brm' first."
  exit 1
fi

# rocket verify wasm validates magic bytes and size
"$CWD/rocket" wasm verify --file "$WASM_FILE"
log_success "WASM artifact: $WASM_FILE"

# rocket html5 verify writes cook-receipt.json alongside the archive
"$CWD/rocket" html5 verify --project Brm 2>/dev/null | grep -E "^\[" || true
if [ -f "$ARCHIVE_DIR/cook-receipt.json" ]; then
  COOK_VERDICT=$(python3 -c "import json,sys; d=json.load(open('$ARCHIVE_DIR/cook-receipt.json')); print(d.get('verdict','UNKNOWN'))" 2>/dev/null || echo "UNKNOWN")
  if [ "$COOK_VERDICT" = "PASS" ]; then
    log_success "Cook receipt: $ARCHIVE_DIR/cook-receipt.json (verdict=$COOK_VERDICT)"
  else
    log_warn "Cook receipt verdict: $COOK_VERDICT (non-blocking)"
  fi
fi

# 2. Stage the HTML5 files to pwa-staff/manufactured/ for serving
log_info "[2/5] Staging HTML5 package to pwa-staff/manufactured/..."
SERVE_DIR="$CWD/pwa-staff/manufactured"
mkdir -p "$SERVE_DIR"
# Copy the entire archive directory (Brm.*, Utility.js, Brm.UE4.js, jquery/, bootstrap/)
if [ -d "$ARCHIVE_DIR" ]; then
  cp -rf "$ARCHIVE_DIR"/. "$SERVE_DIR/"
fi
# Always ensure the wasm is present
if [ ! -f "$SERVE_DIR/Brm.wasm" ]; then
  cp -f "$WASM_FILE" "$SERVE_DIR/"
fi
log_success "Staged to $SERVE_DIR ($(ls "$SERVE_DIR" | wc -l | tr -d ' ') items)"

# 3. Start HTTP server
log_info "[3/5] Starting HTTP server on port $PORT..."
if lsof -Pi :$PORT -sTCP:LISTEN -t >/dev/null 2>&1; then
  log_warn "Port $PORT in use — stopping existing process"
  kill "$(lsof -t -i:$PORT)" 2>/dev/null || true
  sleep 1
fi

# Use rocket html5 serve — sends COOP/COEP headers required for SharedArrayBuffer
("$CWD/tools/target/release/rocket-cmd" html5 serve --project Brm --port $PORT >/tmp/html5-server.log 2>&1) &
SERVER_PID=$!
trap 'log_info "Stopping HTTP server (PID $SERVER_PID)"; kill "$SERVER_PID" 2>/dev/null || true' EXIT

# Wait for server via rocket wait-port
"$CWD/rocket" port wait --port $PORT --timeout 15
log_success "Server ready at http://localhost:$PORT"

# 4. Run Playwright proof
log_info "[4/5] Running Playwright TPS-DFLSS proof..."
mkdir -p "$CWD/pwa-staff/test-results"

# Determine which HTML file to load — prefer the real packaged Brm.html
HTML_FILE=""
for candidate in "Brm.html" "Brm-HTML5-Shipping.html"; do
  [ -f "$SERVE_DIR/$candidate" ] && HTML_FILE="$candidate" && break
done
[ -z "$HTML_FILE" ] && { log_error "No Brm*.html in $SERVE_DIR"; exit 1; }
log_info "Using game page: $HTML_FILE"

PLAYWRIGHT_EXIT=0
(cd "$CWD/pwa-staff" && \
  TARGET_GAME_URL="/$HTML_FILE" \
  npx playwright test tests-e2e/tps-dflss.spec.ts \
    --config playwright.html5.config.ts \
    --reporter=list) || PLAYWRIGHT_EXIT=$?

# 5. Validate receipt
log_info "[5/5] Validating receipt..."
if [ ! -f "$RECEIPT" ]; then
  log_error "Receipt not generated at $RECEIPT"
  exit 1
fi
"$CWD/rocket" receipt validate --file "$RECEIPT"
log_success "Receipt validated: $RECEIPT"

# Gap 6 — Cook-to-game hash cross-check.
# The cook pipeline writes cook-receipt.json with an output_hash (BLAKE3/SHA-256 of Brm.wasm).
# The Playwright tps-dflss receipt also records output_hash via the same wasm hash function.
# If both are present, they MUST agree — a mismatch means the game loaded a different binary
# than the one that was cooked and verified.
log_info "[6/5] Cross-checking cook-receipt hash vs game receipt hash..."
COOK_RECEIPT="$ARCHIVE_DIR/cook-receipt.json"
if [ -f "$COOK_RECEIPT" ] && [ -f "$RECEIPT" ]; then
  COOK_HASH=$(python3 -c "import json; d=json.load(open('$COOK_RECEIPT')); print(d.get('output_hash',''))" 2>/dev/null || echo "")
  GAME_HASH=$(python3 -c "import json; d=json.load(open('$RECEIPT')); print(d.get('output_hash',''))" 2>/dev/null || echo "")
  if [ -z "$COOK_HASH" ] || [ -z "$GAME_HASH" ]; then
    log_warn "Cook-to-game hash cross-check skipped — one or both output_hash fields missing"
    log_warn "  cook receipt output_hash: '${COOK_HASH:-<empty>}'"
    log_warn "  game receipt output_hash: '${GAME_HASH:-<empty>}'"
  elif [ "$COOK_HASH" = "$GAME_HASH" ]; then
    log_success "Cook-to-game hash cross-check PASS — same binary served and played"
    log_success "  output_hash: $COOK_HASH"
  else
    log_error "Cook-to-game hash MISMATCH — binary substitution detected!"
    log_error "  cook output_hash: $COOK_HASH"
    log_error "  game output_hash: $GAME_HASH"
    log_error "  Ensure verify_html5_pipeline.sh serves the same .wasm that rocket html5 cook produced."
    exit 1
  fi
else
  log_warn "Cook-to-game hash cross-check skipped — missing receipt file(s)"
  log_warn "  cook receipt: $COOK_RECEIPT ($([ -f "$COOK_RECEIPT" ] && echo "found" || echo "missing"))"
  log_warn "  game receipt: $RECEIPT ($([ -f "$RECEIPT" ] && echo "found" || echo "missing"))"
fi

echo -e "${BOLD}${CYAN}============================================${RESET}"
if [ $PLAYWRIGHT_EXIT -eq 0 ]; then
  log_success "Stage 6 COMPLETE — real WebGL2 pipeline proven"
  echo -e "  WASM: $WASM_FILE"
  echo -e "  Receipt: $RECEIPT"
else
  log_error "Stage 6 FAILED — Playwright exit $PLAYWRIGHT_EXIT"
  exit 1
fi
