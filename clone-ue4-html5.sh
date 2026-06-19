#!/usr/bin/env bash
set -euo pipefail

: "${GITHUB_TOKEN:?GITHUB_TOKEN must be set — get a token from https://github.com/settings/tokens}"

LOG="$HOME/ue4-build.log"
UE4_DIR="$HOME/ue-4.27-html5-es3"

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*" | tee -a "$LOG"
}

fail() {
    local step="$1"
    echo "BLOCKED: $step FAILED"
    echo "--- last 40 lines of $LOG ---"
    tail -40 "$LOG"
    exit 1
}

log "=== UE4 HTML5 ES3 Setup Start ==="

# Step 1: Clone (skip if already exists)
if [ -d "$UE4_DIR" ]; then
    log "SKIP: $UE4_DIR already exists, skipping clone."
else
    log "Cloning UnrealEngine branch 4.27-html5-es3 into $UE4_DIR ..."
    git clone -b 4.27-html5-es3 --single-branch \
        https://x-access-token:${GITHUB_TOKEN}@github.com/SpeculativeCoder/UnrealEngine.git \
        "$UE4_DIR" >> "$LOG" 2>&1 || fail "git clone"
    log "Clone complete."
fi

# Step 2: Setup.command
log "Running Setup.command ..."
(cd "$UE4_DIR" && ./Setup.command >> "$LOG" 2>&1) || fail "Setup.command"
log "Setup.command complete."

# Step 3: HTML5Setup.sh
log "Running HTML5Setup.sh ..."
(cd "$UE4_DIR/Engine/Platforms/HTML5" && ./HTML5Setup.sh >> "$LOG" 2>&1) || fail "HTML5Setup.sh"
log "HTML5Setup.sh complete."

# Step 4: GenerateProjectFiles.command
log "Running GenerateProjectFiles.command ..."
(cd "$UE4_DIR" && ./GenerateProjectFiles.command >> "$LOG" 2>&1) || fail "GenerateProjectFiles.command"
log "GenerateProjectFiles.command complete."

log "=== UE4 HTML5 ES3 Setup Finished Successfully ==="
echo "UE4 engine is ready at: $UE4_DIR"
echo "Full log: $LOG"
