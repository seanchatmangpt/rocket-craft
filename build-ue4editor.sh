#!/usr/bin/env bash
# Stage 4 — Build UE4Editor
# Delegates to the Rust binary for structured error extraction + stall detection.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TOOLS="$SCRIPT_DIR/tools"
BIN="$TOOLS/target/release/build_ue4"

# Build the Rust binary if needed
if [[ ! -f "$BIN" ]]; then
    echo "[build-ue4editor.sh] Building Rust build_ue4 binary..."
    (cd "$TOOLS" && cargo build --release --bin build_ue4 2>&1)
fi

exec "$BIN" "$@"
