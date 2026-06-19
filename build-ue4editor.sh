#!/usr/bin/env bash
# Stage 4 — Build UE4Editor (Mac / arm64 via Rosetta x86_64)
# Appends to ~/ue4-build.log

set -euo pipefail

LOG="$HOME/ue4-build.log"
ENGINE_ROOT="/Users/sac/ue-4.27-html5-es3"
BUILD_SH="$ENGINE_ROOT/Engine/Build/BatchFiles/Mac/Build.sh"

log() {
    local ts
    ts="$(date '+%Y-%m-%d %H:%M:%S')"
    echo "[$ts] $*" | tee -a "$LOG"
}

fail() {
    local target="$1"
    log "ERROR: $target FAILED"
    echo ""
    echo "BLOCKED: $target FAILED"
    echo ""
    echo "--- last 50 lines of build log ($LOG) ---"
    tail -n 50 "$LOG"
    exit 1
}

log "========================================"
log "STAGE 4 — Build UE4Editor"
log "========================================"

# Gate: engine must be cloned and setup first
log "Checking engine build script exists..."
if [[ ! -f "$BUILD_SH" ]]; then
    log "GATE FAILED: $BUILD_SH not found"
    echo ""
    echo "BLOCKED: Engine not ready — run the clone/setup script first"
    echo "Expected: $BUILD_SH"
    exit 1
fi
log "Gate passed: $BUILD_SH found"

build_target() {
    local target="$1"
    log "----------------------------------------"
    log "Building: $target Mac Development"
    log "----------------------------------------"
    if ! arch -x86_64 /bin/bash "$BUILD_SH" "$target" Mac Development >> "$LOG" 2>&1; then
        fail "$target Mac Development"
    fi
    log "Finished: $target Mac Development"
}

# a. UE4Editor
build_target "UE4Editor"

# b. ShaderCompileWorker
build_target "ShaderCompileWorker"

# c. UnrealPak
build_target "UnrealPak"

log "========================================"
log "All targets built — verifying editor binary"
log "========================================"

EDITOR_BIN="$ENGINE_ROOT/Engine/Binaries/Mac/UE4Editor"

if [[ ! -f "$EDITOR_BIN" ]]; then
    log "VERIFICATION FAILED: $EDITOR_BIN not found"
    echo ""
    echo "BLOCKED: UE4Editor binary missing after build"
    exit 1
fi

SIZE_BYTES=$(stat -f%z "$EDITOR_BIN" 2>/dev/null || stat -c%s "$EDITOR_BIN" 2>/dev/null)
MIN_BYTES=$((100 * 1024 * 1024))  # 100 MB

if [[ "$SIZE_BYTES" -le "$MIN_BYTES" ]]; then
    log "VERIFICATION FAILED: $EDITOR_BIN is only ${SIZE_BYTES} bytes (expected >100 MB)"
    echo ""
    echo "BLOCKED: UE4Editor binary is suspiciously small (${SIZE_BYTES} bytes)"
    exit 1
fi

SIZE_MB=$(( SIZE_BYTES / 1024 / 1024 ))
log "Editor binary: $EDITOR_BIN (${SIZE_MB} MB)"

echo ""
echo "Editor binary: $EDITOR_BIN"
echo "Editor size:   ${SIZE_MB} MB"
echo ""
echo "STAGE 4 COMPLETE — ready for rocket build"
log "STAGE 4 COMPLETE — ready for rocket build"
# Stage 5 auto-chain: run package after successful build
if [ "${AUTO_CHAIN:-0}" = "1" ]; then
    log "Auto-chaining to Stage 5 (package Brm HTML5)..."
    bash /Users/sac/rocket-craft/package-brm-html5.sh
fi
