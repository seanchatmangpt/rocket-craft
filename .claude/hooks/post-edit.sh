#!/usr/bin/env bash
# post-edit.sh — runs lightweight checks after Claude edits a file.
# $1 = absolute path of the edited file.

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
RESET='\033[0m'

FILE="${1:-}"

if [[ -z "$FILE" ]]; then
  echo "[post-edit] No file path provided." >&2
  exit 0
fi

BASENAME="$(basename "$FILE")"
EXT="${BASENAME##*.}"

pass() { echo -e "${GREEN}[post-edit] PASS:${RESET} $1"; }
fail() { echo -e "${RED}[post-edit] FAIL:${RESET} $1"; }
info() { echo -e "${YELLOW}[post-edit] INFO:${RESET} $1"; }

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

info "File edited: $FILE"

# --- Rust source files ---
if [[ "$EXT" == "rs" ]]; then
  WORKSPACE="$(find_cargo_workspace "$(dirname "$FILE")" 2>/dev/null || true)"
  if [[ -n "$WORKSPACE" ]]; then
    info "Running cargo check in $WORKSPACE ..."
    if cargo check --manifest-path "$WORKSPACE/Cargo.toml" 2>&1 | tail -5; then
      pass "cargo check ($WORKSPACE)"
    else
      fail "cargo check ($WORKSPACE)"
    fi
  else
    info "No Cargo.toml found in parent dirs — skipping cargo check."
  fi
fi

# --- TypeScript files ---
if [[ "$EXT" == "ts" || "$EXT" == "tsx" ]]; then
  REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
  PWA_DIR="$REPO_ROOT/pwa-staff"
  if [[ "$FILE" == "$PWA_DIR"* && -f "$PWA_DIR/package.json" ]]; then
    info "Running ESLint in pwa-staff/ ..."
    if (cd "$PWA_DIR" && npm run lint 2>&1 | head -20); then
      pass "lint ($FILE)"
    else
      fail "lint ($FILE)"
    fi
  else
    info "TypeScript file outside pwa-staff/ — no lint configured for this path."
  fi
fi

# --- Python files ---
if [[ "$EXT" == "py" ]]; then
  if command -v python3 &>/dev/null; then
    if python3 -m py_compile "$FILE" 2>&1; then
      pass "py_compile ($FILE)"
    else
      fail "py_compile ($FILE)"
    fi
  else
    info "python3 not found — skipping syntax check."
  fi
fi

# --- Cargo.toml edits ---
if [[ "$BASENAME" == "Cargo.toml" ]]; then
  DIR="$(dirname "$FILE")"
  info "Running cargo check after Cargo.toml edit in $DIR ..."
  if cargo check --manifest-path "$FILE" 2>&1 | tail -5; then
    pass "cargo check ($FILE)"
  else
    fail "cargo check ($FILE)"
  fi
fi
