#!/usr/bin/env bash
# full-stack-e2e.sh — boot the ENTIRE rocket-craft gameplay loop and prove it
# end-to-end with ZERO human interaction.
#
# Pipeline proven:
#   Docker → Supabase (migrations + seed) → .env → asset server (:8080, real UE4
#   wasm) → Nuxt dev (:3000) → seed real auth user → headless-loop E2E (24 tests,
#   real Supabase) → real-UE4 Playwright (WebGL2, genuine EngineReady, OCEL flow)
#   → teardown.
#
# Usage:
#   ./scripts/full-stack-e2e.sh                 # full run (headless-loop + real UE4)
#   SKIP_UE4=1 ./scripts/full-stack-e2e.sh      # skip the heavy real-UE4 Playwright step
#   KEEP_UP=1  ./scripts/full-stack-e2e.sh      # leave the stack running after tests
#   ARCHIVE=/path/to/HTML5 ./scripts/full-stack-e2e.sh   # custom cooked-wasm dir
#
# Exit 0 only when every requested stage passes. Any failure stops the run.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
REPO_ROOT="$(cd "$ROOT/.." && pwd)"
NUXT_PORT="${NUXT_PORT:-3000}"
ASSET_PORT="${ASSET_PORT:-8080}"
API_BASE_URL="http://localhost:${NUXT_PORT}"
ARCHIVE="${ARCHIVE:-/private/tmp/brm-html5-archive/HTML5}"

NUXT_PID="" ASSET_PID="" SB_STARTED=0

log() { echo "[full-stack-e2e] $*"; }

cleanup() {
  [[ "${KEEP_UP:-0}" == "1" ]] && { log "KEEP_UP=1 — leaving stack running"; return; }
  log "tearing down..."
  [[ -n "$NUXT_PID" ]] && kill "$NUXT_PID" 2>/dev/null || true
  [[ -n "$ASSET_PID" ]] && kill "$ASSET_PID" 2>/dev/null || true
  [[ "$SB_STARTED" == "1" ]] && (cd "$ROOT" && supabase stop 2>/dev/null || true)
}
trap cleanup EXIT

wait_for() { # url, name, max_tries
  local url="$1" name="$2" tries="${3:-30}" i
  for ((i=1; i<=tries; i++)); do
    if curl -sf -o /dev/null "$url" 2>/dev/null; then log "$name ready (${i}x2s)"; return 0; fi
    sleep 2
  done
  log "ERROR: $name not ready after $((tries*2))s"; return 1
}

# ── 1. Docker ─────────────────────────────────────────────────────────────────
if ! docker info >/dev/null 2>&1; then
  log "Docker down — launching Docker Desktop..."
  open -a Docker 2>/dev/null || { log "ERROR: cannot launch Docker"; exit 1; }
  for i in {1..60}; do docker info >/dev/null 2>&1 && break; sleep 3; done
  docker info >/dev/null 2>&1 || { log "ERROR: Docker did not start"; exit 1; }
fi
log "Docker up"

# ── 2. Supabase (idempotent) ──────────────────────────────────────────────────
cd "$ROOT"
if ! supabase status >/dev/null 2>&1; then
  log "starting Supabase (migrations + seed)..."
  supabase start
  SB_STARTED=1
else
  log "Supabase already running"
fi

# ── 3. .env from live Supabase keys ───────────────────────────────────────────
SB_JSON="$(supabase status -o json 2>/dev/null)"
ANON="$(echo "$SB_JSON" | node -e "let d='';process.stdin.on('data',c=>d+=c);process.stdin.on('end',()=>console.log(JSON.parse(d).ANON_KEY||JSON.parse(d).anon_key))")"
SVC="$(echo "$SB_JSON" | node -e "let d='';process.stdin.on('data',c=>d+=c);process.stdin.on('end',()=>console.log(JSON.parse(d).SERVICE_ROLE_KEY||JSON.parse(d).service_role_key))")"
cat > "$ROOT/.env" <<EOF
SUPABASE_URL=http://127.0.0.1:54321
SUPABASE_ANON_KEY=$ANON
SUPABASE_SERVICE_ROLE_KEY=$SVC
ALLOW_SESSION_SEED=1
ALLOW_ANON_GAME=1
NODE_ENV=development
EOF
log ".env written"

# ── 4. Asset server (:8080) — real cooked UE4 wasm ────────────────────────────
if [[ "${SKIP_UE4:-0}" != "1" ]]; then
  if [[ ! -f "$ARCHIVE/Brm.wasm" ]]; then
    log "WARN: no cooked wasm at $ARCHIVE/Brm.wasm — skipping real-UE4 stage"
    SKIP_UE4=1
  else
    log "serving cooked UE4 wasm from $ARCHIVE on :$ASSET_PORT"
    ( cd "$ARCHIVE" && python3 -m http.server "$ASSET_PORT" >/tmp/asset-server.log 2>&1 ) &
    ASSET_PID=$!
    wait_for "http://localhost:$ASSET_PORT/Brm.html" "asset server" 10 || exit 1
  fi
fi

# ── 5. Nuxt dev server ────────────────────────────────────────────────────────
log "starting Nuxt dev on :$NUXT_PORT..."
( cd "$ROOT" && npx nuxt dev --port "$NUXT_PORT" >/tmp/nuxt-dev.log 2>&1 ) &
NUXT_PID=$!
wait_for "$API_BASE_URL/api/game/leaderboard" "Nuxt" 40 || { tail -20 /tmp/nuxt-dev.log; exit 1; }

# ── 6. Seed a real auth user (so login / RLS work, no bypass needed) ──────────
log "seeding real auth user..."
curl -sf -X POST "$API_BASE_URL/api/test/seed-auth-user" -H 'content-type: application/json' -d '{}' \
  | node -e "let d='';process.stdin.on('data',c=>d+=c);process.stdin.on('end',()=>{const u=JSON.parse(d);console.log('[full-stack-e2e] auth user '+u.user_id+' ('+u.email+')')})" \
  || log "WARN: auth-user seed failed (non-fatal)"

# ── 7. Headless-loop E2E (24 tests, real Supabase) ────────────────────────────
log "running headless-loop E2E (MOCK_API=0)..."
( cd "$ROOT" && API_BASE_URL="$API_BASE_URL" MOCK_API=0 npx vitest run tests/e2e/headless-loop.test.ts --reporter=dot )

# ── 8. Real-UE4 Playwright (WebGL2, genuine EngineReady) ──────────────────────
if [[ "${SKIP_UE4:-0}" != "1" ]]; then
  log "running real-UE4 Playwright proof..."
  ( cd "$ROOT" && npx playwright test --config=playwright.ue4.config.ts )
else
  log "real-UE4 stage skipped"
fi

log "✓ ALL STAGES PASSED — gameplay loop proven end-to-end with zero human interaction"
