#!/usr/bin/env bash
# test-ci.sh — Full automated gameplay loop CI harness.
#
# Runs the complete end-to-end pipeline without human interaction:
#   1. Start local Supabase
#   2. Apply all migrations
#   3. Start Nuxt dev server
#   4. Run headless-loop.test.ts (MOCK_API=0) against real endpoints
#   5. Assert all 14 steps pass
#   6. Tear down
#
# Usage:
#   ./scripts/test-ci.sh              # full CI run
#   SKIP_SUPABASE=1 ./scripts/test-ci.sh  # skip Supabase start (already running)
#   SKIP_NUXT=1 ./scripts/test-ci.sh      # skip Nuxt start (already running on :3000)
#
# Environment:
#   SUPABASE_URL              — default http://localhost:54321
#   SUPABASE_ANON_KEY         — default from supabase status
#   SUPABASE_SERVICE_ROLE_KEY — default from supabase status
#   NUXT_PORT                 — default 3000
#   API_BASE_URL              — default http://localhost:${NUXT_PORT}

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
NUXT_PORT="${NUXT_PORT:-3000}"
API_BASE_URL="${API_BASE_URL:-http://localhost:${NUXT_PORT}}"

NUXT_PID=""
SB_STARTED=0

cleanup() {
  echo ""
  echo "[test-ci] Cleaning up..."
  if [[ -n "$NUXT_PID" ]]; then
    kill "$NUXT_PID" 2>/dev/null || true
    echo "[test-ci] Stopped Nuxt (pid $NUXT_PID)"
  fi
  if [[ "$SB_STARTED" == "1" ]]; then
    supabase stop --no-backup 2>/dev/null || true
    echo "[test-ci] Stopped Supabase"
  fi
}
trap cleanup EXIT

# ── Step 1: Local Supabase ────────────────────────────────────────────────────
if [[ "${SKIP_SUPABASE:-0}" != "1" ]]; then
  echo "[test-ci] Starting local Supabase..."
  cd "$ROOT"
  supabase start
  SB_STARTED=1

  # Capture keys from supabase status
  STATUS=$(supabase status 2>/dev/null)
  export SUPABASE_URL="${SUPABASE_URL:-http://localhost:54321}"
  export SUPABASE_ANON_KEY="${SUPABASE_ANON_KEY:-$(echo "$STATUS" | grep 'anon key' | awk '{print $NF}')}"
  export SUPABASE_SERVICE_ROLE_KEY="${SUPABASE_SERVICE_ROLE_KEY:-$(echo "$STATUS" | grep 'service_role key' | awk '{print $NF}')}"
  echo "[test-ci] Supabase up at $SUPABASE_URL"
fi

# ── Step 2: Apply migrations ──────────────────────────────────────────────────
echo "[test-ci] Applying migrations..."
cd "$ROOT"
supabase db push 2>/dev/null || supabase migration up 2>/dev/null || {
  echo "[test-ci] WARNING: migration command failed — may already be applied"
}

# ── Step 3: Start Nuxt dev server ─────────────────────────────────────────────
if [[ "${SKIP_NUXT:-0}" != "1" ]]; then
  echo "[test-ci] Starting Nuxt dev server on port $NUXT_PORT..."
  cd "$ROOT"
  NUXT_PORT="$NUXT_PORT" \
  SUPABASE_URL="$SUPABASE_URL" \
  SUPABASE_ANON_KEY="$SUPABASE_ANON_KEY" \
  SUPABASE_SERVICE_ROLE_KEY="$SUPABASE_SERVICE_ROLE_KEY" \
    npx nuxt dev --port "$NUXT_PORT" &
  NUXT_PID=$!

  # Wait for Nuxt to be ready (max 60s)
  echo "[test-ci] Waiting for Nuxt at $API_BASE_URL..."
  for i in $(seq 1 60); do
    if curl -sf "$API_BASE_URL/api/game/leaderboard" > /dev/null 2>&1; then
      echo "[test-ci] Nuxt ready after ${i}s"
      break
    fi
    if [[ $i == 60 ]]; then
      echo "[test-ci] ERROR: Nuxt did not start within 60s"
      exit 1
    fi
    sleep 1
  done
fi

# ── Step 4: Run headless loop tests ──────────────────────────────────────────
echo ""
echo "[test-ci] ════════════════════════════════════════════"
echo "[test-ci] Running headless gameplay loop E2E tests"
echo "[test-ci] API_BASE_URL=$API_BASE_URL"
echo "[test-ci] ════════════════════════════════════════════"
echo ""

cd "$ROOT"
API_BASE_URL="$API_BASE_URL" \
MOCK_API=0 \
  npx vitest run tests/e2e/headless-loop.test.ts --reporter=verbose 2>&1

EXIT_CODE=$?

# ── Step 5: Assert result ─────────────────────────────────────────────────────
echo ""
if [[ $EXIT_CODE -eq 0 ]]; then
  echo "[test-ci] ✓ All headless loop tests PASSED"
else
  echo "[test-ci] ✗ Headless loop tests FAILED (exit $EXIT_CODE)"
fi

exit $EXIT_CODE
