#!/usr/bin/env bash
set -euo pipefail

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
RESET='\033[0m'

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
EXPECTED_BRANCH="claude/inspiring-hamilton-jfww9e"

echo ""
echo -e "${BOLD}${CYAN}========================================${RESET}"
echo -e "${BOLD}${CYAN}  ROCKET-CRAFT  |  Claude Code Session  ${RESET}"
echo -e "${BOLD}${CYAN}========================================${RESET}"

# Branch check
CURRENT_BRANCH="$(git -C "$REPO_ROOT" rev-parse --abbrev-ref HEAD 2>/dev/null || echo 'unknown')"
echo -e "\n${BOLD}Branch:${RESET} $CURRENT_BRANCH"
if [[ "$CURRENT_BRANCH" != "$EXPECTED_BRANCH" ]]; then
  echo -e "${YELLOW}  WARNING: Expected branch '$EXPECTED_BRANCH'${RESET}"
else
  echo -e "${GREEN}  Branch OK${RESET}"
fi

# Toolchain checks
echo -e "\n${BOLD}Toolchain:${RESET}"

if command -v rustup &>/dev/null; then
  RUST_VERSION="$(rustup show active-toolchain 2>/dev/null | awk '{print $1}' || rustc --version 2>/dev/null | awk '{print $2}')"
  echo -e "  ${GREEN}Rust:${RESET} $RUST_VERSION"
else
  echo -e "  ${RED}Rust: rustup not found${RESET}"
fi

if command -v node &>/dev/null; then
  echo -e "  ${GREEN}Node.js:${RESET} $(node --version)"
else
  echo -e "  ${RED}Node.js: not found${RESET}"
fi

if command -v python3 &>/dev/null; then
  echo -e "  ${GREEN}Python3:${RESET} $(python3 --version 2>&1)"
else
  echo -e "  ${RED}Python3: not found${RESET}"
fi

# Git status
echo -e "\n${BOLD}Git status:${RESET}"
GIT_STATUS="$(git -C "$REPO_ROOT" status --short 2>/dev/null)"
if [[ -z "$GIT_STATUS" ]]; then
  echo -e "  ${GREEN}Working tree clean${RESET}"
else
  CHANGE_COUNT="$(echo "$GIT_STATUS" | wc -l | tr -d ' ')"
  echo -e "  ${YELLOW}$CHANGE_COUNT uncommitted change(s):${RESET}"
  echo "$GIT_STATUS" | head -10 | sed 's/^/    /'
  [[ "$(echo "$GIT_STATUS" | wc -l)" -gt 10 ]] && echo "    ... (truncated)"
fi

# Environment checks
echo -e "\n${BOLD}Environment:${RESET}"
if [[ -n "${UE4_ROOT:-}" ]]; then
  echo -e "  ${GREEN}UE4_ROOT:${RESET} $UE4_ROOT"
else
  echo -e "  ${YELLOW}UE4_ROOT: not set (Unreal Engine features unavailable)${RESET}"
fi

if command -v blender &>/dev/null; then
  echo -e "  ${GREEN}Blender:${RESET} $(command -v blender)"
elif [[ -n "${BLENDER_PATH:-}" ]]; then
  echo -e "  ${GREEN}BLENDER_PATH:${RESET} $BLENDER_PATH"
else
  echo -e "  ${YELLOW}Blender: not found in PATH and BLENDER_PATH not set (asset pipeline limited)${RESET}"
fi

# Asset validation
echo -e "\n${BOLD}Asset validation:${RESET}"
if [[ -f "$REPO_ROOT/validate-assets.py" ]]; then
  if python3 "$REPO_ROOT/validate-assets.py" 2>&1 | tail -5; then
    echo -e "  ${GREEN}validate-assets.py passed${RESET}"
  else
    echo -e "  ${YELLOW}validate-assets.py reported issues (see above)${RESET}"
  fi
else
  echo -e "  ${YELLOW}validate-assets.py not found${RESET}"
fi

# Rocket CLI
echo -e "\n${BOLD}Rocket CLI:${RESET}"
if [[ -x "$REPO_ROOT/rocket" ]]; then
  "$REPO_ROOT/rocket" --help 2>/dev/null | head -6 | sed 's/^/  /' || echo "  (rocket --help failed)"
else
  echo -e "  ${YELLOW}rocket CLI not built${RESET}"
fi

echo ""
echo -e "${GREEN}${BOLD}Ready at: $(date)${RESET}"
echo -e "${CYAN}========================================${RESET}"
echo ""
