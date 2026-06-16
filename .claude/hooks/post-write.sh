#!/usr/bin/env bash
# post-write.sh — runs checks after Claude writes/creates a new file.
# $1 = absolute path of the written file.

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
RESET='\033[0m'

FILE="${1:-}"

if [[ -z "$FILE" ]]; then
  echo "[post-write] No file path provided." >&2
  exit 0
fi

BASENAME="$(basename "$FILE")"
EXT="${BASENAME##*.}"

pass() { echo -e "${GREEN}[post-write] PASS:${RESET} $1"; }
fail() { echo -e "${RED}[post-write] FAIL:${RESET} $1"; }
info() { echo -e "${YELLOW}[post-write] INFO:${RESET} $1"; }

info "Created: $FILE at $(date)"

# Locate nearest Cargo.toml walking up from $DIR
find_cargo_workspace() {
  local dir="$1"
  while [[ "$dir" != "/" ]]; do
    if [[ -f "$dir/Cargo.toml" ]]; then
      echo "$dir"
      return 0
    fi
    dir="$(dirname "$dir")"
  done
  return 1
}

# --- New Rust source file ---
if [[ "$EXT" == "rs" ]]; then
  WORKSPACE="$(find_cargo_workspace "$(dirname "$FILE")" 2>/dev/null || true)"
  if [[ -n "$WORKSPACE" ]]; then
    info "New Rust file detected — running cargo check in $WORKSPACE ..."
    if cargo check --manifest-path "$WORKSPACE/Cargo.toml" 2>&1 | tail -5; then
      pass "cargo check ($WORKSPACE)"
    else
      fail "cargo check ($WORKSPACE) — new file may have errors"
    fi
  else
    info "No Cargo.toml found in parent dirs — skipping cargo check."
  fi
fi

# --- New Cargo.toml ---
if [[ "$BASENAME" == "Cargo.toml" ]]; then
  info "New Cargo.toml detected — running cargo metadata to validate ..."
  if cargo metadata --no-deps --manifest-path "$FILE" 2>&1 | tail -5; then
    pass "cargo metadata ($FILE)"
  else
    fail "cargo metadata ($FILE) — Cargo.toml may be malformed"
  fi
fi

# --- New Python file ---
if [[ "$EXT" == "py" ]]; then
  if command -v python3 &>/dev/null; then
    info "New Python file — running syntax check ..."
    if python3 -m py_compile "$FILE" 2>&1; then
      pass "py_compile ($FILE)"
    else
      fail "py_compile ($FILE) — syntax error in new file"
    fi
  else
    info "python3 not found — skipping syntax check."
  fi
fi
