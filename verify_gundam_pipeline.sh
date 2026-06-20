#!/usr/bin/env bash
# verify_gundam_pipeline.sh — Stage 6: Serve real Brm.wasm + Playwright proof + receipt for Gundam Factory Walkthrough.
# Usage: ./verify_gundam_pipeline.sh [archive_dir]
# archive_dir defaults to /tmp/brm-html5-archive/HTML5

set -euo pipefail

CWD="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
_BASE="${1:-/tmp/brm-html5-archive}"
if [ -d "$_BASE/HTML5" ] && ls "$_BASE/HTML5/"*.wasm 2>/dev/null | head -1 | grep -q .; then
  ARCHIVE_DIR="$_BASE/HTML5"
else
  ARCHIVE_DIR="$_BASE"
fi
PORT=8080
RECEIPT="$CWD/pwa-staff/test-results/gundam-factory-playwright-receipt.json"

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
echo -e "${BOLD}${CYAN}  Rocket Craft — Gundam E2E Walkthrough Proof ${RESET}"
echo -e "${BOLD}${CYAN}============================================${RESET}"

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
  log_error "No Brm.wasm found."
  exit 1
fi

"$CWD/rocket" wasm verify --file "$WASM_FILE"
log_success "WASM artifact: $WASM_FILE"

# 2. Stage the HTML5 files to pwa-staff/manufactured/ for serving
log_info "[2/5] Staging HTML5 package to pwa-staff/manufactured/..."
SERVE_DIR="$CWD/pwa-staff/manufactured"
mkdir -p "$SERVE_DIR"
if [ -d "$ARCHIVE_DIR" ]; then
  cp -rf "$ARCHIVE_DIR"/. "$SERVE_DIR/"
fi
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

("$CWD/tools/target/release/rocket-cmd" html5 serve --project Brm --port $PORT >/tmp/html5-server.log 2>&1) &
SERVER_PID=$!
trap 'log_info "Stopping HTTP server (PID $SERVER_PID)"; kill "$SERVER_PID" 2>/dev/null || true' EXIT

"$CWD/rocket" port wait --port $PORT --timeout 15
log_success "Server ready at http://localhost:$PORT"

# 4. Run Playwright proof
log_info "[4/5] Running Playwright Gundam Walkthrough proof..."
mkdir -p "$CWD/pwa-staff/test-results"

HTML_FILE=""
for candidate in "Brm.html" "Brm-HTML5-Shipping.html"; do
  [ -f "$SERVE_DIR/$candidate" ] && HTML_FILE="$candidate" && break
done
[ -z "$HTML_FILE" ] && { log_error "No Brm*.html in $SERVE_DIR"; exit 1; }
log_info "Using game page: $HTML_FILE"

PLAYWRIGHT_EXIT=0
(cd "$CWD/pwa-staff" && \
  TARGET_GAME_URL="/$HTML_FILE" \
  npx playwright test tests-e2e/gundam_factory_walkthrough_projection.spec.ts \
    --config playwright.gundam.config.ts \
    --reporter=list) || PLAYWRIGHT_EXIT=$?

# 5. Validate receipt
log_info "[5/5] Validating receipt..."
if [ ! -f "$RECEIPT" ]; then
  log_error "Receipt not generated at $RECEIPT"
  exit 1
fi
"$CWD/rocket" receipt validate --file "$RECEIPT"
log_success "Receipt validated: $RECEIPT"

echo -e "${BOLD}${CYAN}============================================${RESET}"
if [ $PLAYWRIGHT_EXIT -eq 0 ]; then
  log_success "Gundam Walkthrough COMPLETE — real WebGL2 pipeline proven"
  echo -e "  WASM: $WASM_FILE"
  echo -e "  Receipt: $RECEIPT"
else
  log_error "Gundam Walkthrough FAILED — Playwright exit $PLAYWRIGHT_EXIT"
  exit 1
fi
