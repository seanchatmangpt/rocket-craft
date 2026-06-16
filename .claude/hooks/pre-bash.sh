#!/usr/bin/env bash
# pre-bash.sh — blocks or warns on dangerous commands before Claude executes them.
# Claude Code passes the command via CLAUDE_TOOL_INPUT_COMMAND env var; $1 is a fallback.

RED='\033[0;31m'
YELLOW='\033[0;33m'
RESET='\033[0m'

CMD="${CLAUDE_TOOL_INPUT_COMMAND:-${1:-}}"

if [[ -z "$CMD" ]]; then
  exit 0
fi

block() {
  echo -e "${RED}[pre-bash] BLOCKED: $1${RESET}" >&2
  exit 2
}

warn() {
  echo -e "${YELLOW}[pre-bash] WARNING: $1${RESET}" >&2
}

# --- Hard blocks ---

# rm -rf / or rm -rf ~ (and variations with env vars or quotes)
if echo "$CMD" | grep -qE 'rm\s+(-[a-zA-Z]*f[a-zA-Z]*\s+|--force\s+).*(-[a-zA-Z]*r[a-zA-Z]*\s+|--recursive\s+).*(/\s*$|~\s*$|/\s*[;&|]|~\s*[;&|])|rm\s+(-[a-zA-Z]*r[a-zA-Z]*\s+|--recursive\s+).*(-[a-zA-Z]*f[a-zA-Z]*\s+|--force\s+).*(\/\s*$|~\s*$|\/\s*[;&|]|~\s*[;&|])'; then
  block "Recursive forced removal of root or home directory detected: $CMD"
fi
# Simple pattern catch for rm -rf / style
if echo "$CMD" | grep -qP 'rm\s+\S*r\S*f\S*\s+[/~](\s|$)|rm\s+\S*f\S*r\S*\s+[/~](\s|$)'; then
  block "Recursive forced removal of root or home directory detected: $CMD"
fi

# git push --force to main or master
if echo "$CMD" | grep -qE 'git\s+push.*(--force|-f)\s.*(main|master)|git\s+push.*(main|master).*(--force|-f)'; then
  block "Force push to main/master is not allowed: $CMD"
fi
# git push --force-with-lease is riskier than --force but still block for main/master
if echo "$CMD" | grep -qE 'git\s+push.*--force-with-lease.*(main|master)|git\s+push.*(main|master).*--force-with-lease'; then
  block "Force push (--force-with-lease) to main/master is not allowed: $CMD"
fi

# --- Warnings (non-blocking) ---

# git reset --hard
if echo "$CMD" | grep -qE 'git\s+reset\s+--hard'; then
  warn "git reset --hard detected — this discards uncommitted changes permanently: $CMD"
fi

# cargo clean with no target (in a large multi-workspace repo this can take minutes)
if echo "$CMD" | grep -qE '^\s*cargo\s+clean\s*$'; then
  warn "cargo clean with no arguments in a large workspace — this may take several minutes to rebuild everything."
fi

# SQL DROP TABLE
if echo "$CMD" | grep -qiE '\bDROP\s+TABLE\b'; then
  warn "DROP TABLE detected in command — ensure this is intentional: $CMD"
fi

# SQL DELETE FROM without WHERE
if echo "$CMD" | grep -qiE '\bDELETE\s+FROM\b'; then
  if ! echo "$CMD" | grep -qiE '\bWHERE\b'; then
    warn "DELETE FROM without WHERE clause detected — this will delete all rows: $CMD"
  fi
fi

exit 0
