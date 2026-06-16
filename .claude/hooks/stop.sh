#!/usr/bin/env bash
set -euo pipefail

GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
RESET='\033[0m'

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

echo ""
echo -e "${CYAN}${BOLD}==============================${RESET}"
echo -e "${CYAN}${BOLD}  Session Ended — rocket-craft ${RESET}"
echo -e "${CYAN}${BOLD}==============================${RESET}"

# Branch
BRANCH="$(git -C "$REPO_ROOT" rev-parse --abbrev-ref HEAD 2>/dev/null || echo 'unknown')"
echo -e "\n${BOLD}Branch:${RESET} $BRANCH"

# Git status
GIT_STATUS="$(git -C "$REPO_ROOT" status --short 2>/dev/null || true)"
echo -e "\n${BOLD}Git status:${RESET}"
if [[ -z "$GIT_STATUS" ]]; then
  echo -e "  ${GREEN}Working tree clean — nothing to commit.${RESET}"
else
  CHANGE_COUNT="$(echo "$GIT_STATUS" | wc -l | tr -d ' ')"
  echo -e "  ${YELLOW}$CHANGE_COUNT uncommitted change(s):${RESET}"
  echo "$GIT_STATUS" | sed 's/^/    /'
  echo ""
  echo -e "  ${YELLOW}Reminder: commit and push your work before ending the session.${RESET}"
  echo -e "  ${YELLOW}  git add -p && git commit -m \"<message>\" && git push${RESET}"
fi

# Rust files check
RUST_MODIFIED="$(echo "$GIT_STATUS" | grep -E '\.rs$' || true)"
if [[ -n "$RUST_MODIFIED" ]]; then
  echo ""
  echo -e "  ${YELLOW}Rust files were modified — consider running:${RESET}"
  echo -e "  ${YELLOW}  cargo test${RESET}"
fi

echo ""
echo -e "${BOLD}Session ended at:${RESET} $(date)"
echo -e "${CYAN}==============================${RESET}"
echo ""
