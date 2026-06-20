#!/usr/bin/env bash
# =============================================================================
# scripts/cloud-setup.sh — Bootstrap Rocket Craft in cloud / CI environments
#
# Safe to run multiple times (idempotent).
# Does NOT require Unreal Engine 4 or Blender; marks them as optional.
#
# Usage:
#   bash scripts/cloud-setup.sh
#   ROCKET_CLOUD=1 bash scripts/cloud-setup.sh   # suppress UE4 warnings
# =============================================================================
set -euo pipefail

# ---------------------------------------------------------------------------
# Colour support
# ---------------------------------------------------------------------------
if [ -t 1 ] && [ "${NO_COLOR:-}" = "" ]; then
    BOLD="\033[1m"
    RED="\033[31m"
    GREEN="\033[32m"
    YELLOW="\033[33m"
    BLUE="\033[34m"
    RESET="\033[0m"
else
    BOLD="" RED="" GREEN="" YELLOW="" BLUE="" RESET=""
fi

ok()   { echo -e "  ${GREEN}${BOLD}[OK]${RESET}      $*"; }
warn() { echo -e "  ${YELLOW}${BOLD}[WARN]${RESET}    $*"; }
miss() { echo -e "  ${RED}${BOLD}[MISSING]${RESET} $*"; }
info() { echo -e "${BLUE}${BOLD}[INFO]${RESET} $*"; }

# ---------------------------------------------------------------------------
# OS detection
# ---------------------------------------------------------------------------
OS="unknown"
case "$(uname -s)" in
    Linux*)  OS="linux" ;;
    Darwin*) OS="macos" ;;
esac
info "Detected OS: ${OS}"

# ---------------------------------------------------------------------------
# Resolve project root (the directory containing this script's parent)
# ---------------------------------------------------------------------------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
info "Project root: ${PROJECT_ROOT}"

# ---------------------------------------------------------------------------
# Tracking
# ---------------------------------------------------------------------------
MISSING=()
OPTIONAL_MISSING=()

# ---------------------------------------------------------------------------
# Helper: check a command exists
# ---------------------------------------------------------------------------
has_cmd() { command -v "$1" &>/dev/null; }

# ===========================================================================
# 1. REQUIRED TOOLS
# ===========================================================================
echo ""
info "Checking required tools..."

# --- rustup -----------------------------------------------------------------
if has_cmd rustup; then
    RUSTUP_VER="$(rustup --version 2>&1 | head -1)"
    ok "rustup: ${RUSTUP_VER}"
else
    miss "rustup not found. Install from https://rustup.rs"
    MISSING+=("rustup")
fi

# --- cargo ------------------------------------------------------------------
if has_cmd cargo; then
    CARGO_VER="$(cargo --version 2>&1)"
    ok "cargo: ${CARGO_VER}"
else
    miss "cargo not found (install rustup first)"
    MISSING+=("cargo")
fi

# --- node -------------------------------------------------------------------
if has_cmd node; then
    NODE_VER="$(node --version 2>&1)"
    ok "node: ${NODE_VER}"
    # Warn if not Node 20.x (the version pwa-staff targets)
    NODE_MAJOR="$(node --version | sed 's/v\([0-9]*\).*/\1/')"
    if [ "${NODE_MAJOR}" -lt 20 ]; then
        warn "node ${NODE_VER} found but pwa-staff requires >= 20. Consider upgrading."
    fi
else
    miss "node not found. Install Node.js 20.x from https://nodejs.org"
    MISSING+=("node")
fi

# --- npm -------------------------------------------------------------------
if has_cmd npm; then
    NPM_VER="$(npm --version 2>&1)"
    ok "npm: ${NPM_VER}"
else
    miss "npm not found (should ship with Node.js)"
    MISSING+=("npm")
fi

# ===========================================================================
# 2. RUST TOOLCHAIN COMPONENTS
# ===========================================================================
echo ""
info "Installing / verifying Rust toolchain components..."

if has_cmd rustup; then
    # rustfmt
    if rustup component list --installed 2>/dev/null | grep -q "^rustfmt"; then
        ok "rustfmt already installed"
    else
        info "Installing rustfmt..."
        rustup component add rustfmt
        ok "rustfmt installed"
    fi

    # clippy
    if rustup component list --installed 2>/dev/null | grep -q "^clippy"; then
        ok "clippy already installed"
    else
        info "Installing clippy..."
        rustup component add clippy
        ok "clippy installed"
    fi
else
    warn "Skipping Rust component install (rustup not available)"
fi

# ===========================================================================
# 3. PWA DEPENDENCIES
# ===========================================================================
echo ""
info "Installing pwa-staff npm dependencies..."

PWA_DIR="${PROJECT_ROOT}/pwa-staff"
if [ -d "${PWA_DIR}" ] && [ -f "${PWA_DIR}/package-lock.json" ]; then
    (cd "${PWA_DIR}" && npm ci)
    ok "pwa-staff: npm ci complete"
else
    warn "pwa-staff directory or package-lock.json not found; skipping npm ci"
fi

# ===========================================================================
# 4. OPTIONAL TOOLS (UE4, Blender)
# ===========================================================================
echo ""
info "Checking optional tools..."

# --- Unreal Engine 4 -------------------------------------------------------
UE4_ROOT="${UE4_ROOT:-}"
if [ -n "${UE4_ROOT}" ] && [ -d "${UE4_ROOT}" ]; then
    ok "UE4_ROOT set and exists: ${UE4_ROOT}"
else
    warn "UE4_ROOT not set or directory missing (required for UE4 builds and html5 pipeline)"
    OPTIONAL_MISSING+=("UE4_ROOT")
fi

# --- Blender ---------------------------------------------------------------
BLENDER_PATH="${BLENDER_PATH:-}"
if [ -n "${BLENDER_PATH}" ] && [ -x "${BLENDER_PATH}" ]; then
    ok "BLENDER_PATH set and executable: ${BLENDER_PATH}"
elif has_cmd blender; then
    BLENDER_BIN="$(command -v blender)"
    ok "blender found on PATH: ${BLENDER_BIN}"
else
    warn "Blender not found (only required by asset-pipeline — set BLENDER_PATH or add to PATH)"
    OPTIONAL_MISSING+=("blender")
fi

# --- Docker (for local Supabase) -------------------------------------------
if has_cmd docker; then
    DOCKER_VER="$(docker --version 2>&1)"
    ok "docker: ${DOCKER_VER}"
else
    warn "docker not found (only required for local Supabase via 'supabase start')"
    OPTIONAL_MISSING+=("docker")
fi

# --- supabase CLI -----------------------------------------------------------
if has_cmd supabase; then
    SB_VER="$(supabase --version 2>&1 | head -1)"
    ok "supabase CLI: ${SB_VER}"
else
    warn "supabase CLI not found (only required for local Supabase dev)"
    OPTIONAL_MISSING+=("supabase-cli")
fi

# --- Python 3 (for scripts/ and Blender .py) -------------------------------
if has_cmd python3; then
    PY_VER="$(python3 --version 2>&1)"
    ok "python3: ${PY_VER}"
else
    warn "python3 not found (required by scripts/pm4py_conformance.py and Blender Python scripts)"
    OPTIONAL_MISSING+=("python3")
fi

# ===========================================================================
# 5. SUMMARY
# ===========================================================================
echo ""
echo -e "${BOLD}========================================${RESET}"
echo -e "${BOLD} Cloud Setup — Summary${RESET}"
echo -e "${BOLD}========================================${RESET}"

if [ "${#MISSING[@]}" -eq 0 ]; then
    echo -e "${GREEN}${BOLD}All required tools are present.${RESET}"
else
    echo -e "${RED}${BOLD}Required tools missing (${#MISSING[@]}):${RESET}"
    for item in "${MISSING[@]}"; do
        miss "${item}"
    done
fi

if [ "${#OPTIONAL_MISSING[@]}" -gt 0 ]; then
    echo -e "${YELLOW}${BOLD}Optional tools not found (${#OPTIONAL_MISSING[@]}):${RESET}"
    for item in "${OPTIONAL_MISSING[@]}"; do
        warn "${item}"
    done
fi

echo ""
if [ "${#MISSING[@]}" -gt 0 ]; then
    echo -e "${RED}${BOLD}Setup incomplete — install missing required tools and re-run.${RESET}"
    exit 1
else
    echo -e "${GREEN}${BOLD}Setup complete. You can now run:${RESET}"
    echo "  ./rocket build     # Build UE4 projects (requires UE4_ROOT)"
    echo "  ./rocket test      # Run all Rust tests + asset validation"
    echo "  ./rocket pwa lint  # Lint pwa-staff"
    echo "  cd pwa-staff && npm test   # Run PWA unit tests"
fi

# ===========================================================================
# 6. DIRENV / .envrc INTEGRATION
# ===========================================================================
echo ""
info "Checking direnv integration..."

if has_cmd direnv; then
    DIRENV_VER="$(direnv version 2>&1)"
    ok "direnv: ${DIRENV_VER}"
    if [ ! -f "${PROJECT_ROOT}/.envrc" ]; then
        info "No .envrc found — generating one with ./rocket env generate-envrc ..."
        if [ -x "${PROJECT_ROOT}/rocket" ]; then
            (cd "${PROJECT_ROOT}" && ./rocket env generate-envrc) \
                && ok ".envrc generated. Run 'direnv allow' in ${PROJECT_ROOT} to activate." \
                || warn "Failed to generate .envrc via rocket CLI."
        else
            warn "rocket binary not found at ${PROJECT_ROOT}/rocket. Build it first with: cd tools && cargo build --release"
        fi
    else
        ok ".envrc already exists at ${PROJECT_ROOT}/.envrc"
        info "Run 'direnv allow' if you haven't already."
    fi
else
    warn "direnv not found. Install from https://direnv.net to enable automatic env loading."
    info "Once installed, run: ./rocket env generate-envrc && direnv allow"
fi
