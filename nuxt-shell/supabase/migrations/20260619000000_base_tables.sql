-- Base tables for the rocket-craft OCEL game session proof pipeline.
--
-- Schema design principles:
--   - game_sessions  : one row per browser or CLI cook session
--   - ocel_events    : tamper-evident hash-chain event log (SHA-256 chaining)
--   - game_receipts  : verdicts (PASS/FAIL) written by browser or Rust CLI
--   - players        : pilot identities (linked to Supabase auth.users)
--   - leaderboard    : auto-maintained by trigger on game_receipts
--
-- Pattern: ~/dashboard.bak/supabase/migrations/002_events_raw.sql (hash chain)

-- ── players ───────────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS players (
  id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  auth_user_id UUID UNIQUE REFERENCES auth.users(id) ON DELETE CASCADE,
  username     TEXT,
  high_score   BIGINT NOT NULL DEFAULT 0,
  created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS players_auth_user_id_idx ON players(auth_user_id);

-- ── game_sessions ─────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS game_sessions (
  id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  -- NULL for anonymous browser sessions and Rust CLI cook sessions
  player_id          UUID REFERENCES players(id) ON DELETE SET NULL,
  session_started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  session_ended_at   TIMESTAMPTZ,
  is_alive           BOOLEAN NOT NULL DEFAULT false,
  ocel_event_count   INTEGER NOT NULL DEFAULT 0,
  -- 'real_ue4' | 'rocket_cli' | 'synthetic' | 'unknown'
  engine_source      TEXT NOT NULL DEFAULT 'unknown',
  -- SHA-256 hex of the final session receipt
  receipt_hash       TEXT,
  -- Free-form metadata: browser UA, project name, archive path, etc.
  metadata           JSONB NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS game_sessions_player_id_idx    ON game_sessions(player_id);
CREATE INDEX IF NOT EXISTS game_sessions_started_at_idx   ON game_sessions(session_started_at DESC);
CREATE INDEX IF NOT EXISTS game_sessions_is_alive_idx     ON game_sessions(is_alive) WHERE is_alive = true;

-- ── ocel_events ───────────────────────────────────────────────────────────────
-- SHA-256 hash chain: each event commits the previous hash, making the log
-- tamper-evident and replayable by pm4py for process conformance checking.

CREATE TABLE IF NOT EXISTS ocel_events (
  id           BIGSERIAL PRIMARY KEY,
  session_id   UUID REFERENCES game_sessions(id) ON DELETE CASCADE,
  -- OCEL 2.0 activity name: 'GameSessionStarted', 'FrameRendered', 'InputAdmitted', etc.
  activity     TEXT NOT NULL,
  timestamp_ms BIGINT NOT NULL,
  -- Array of object IDs this event relates to (OCEL object-centric model)
  object_refs  TEXT[] NOT NULL DEFAULT '{}',
  -- Arbitrary event attributes (intent type, value, source, etc.)
  attributes   JSONB NOT NULL DEFAULT '{}',
  -- Hash chain: SHA-256(prev_hash || activity || timestamp_ms || attributes)
  prev_hash    TEXT,
  event_hash   TEXT NOT NULL,
  -- Monotonic sequence within the session for ordering
  seq          INTEGER NOT NULL,
  created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),

  UNIQUE (session_id, seq)
);

CREATE INDEX IF NOT EXISTS ocel_events_session_id_idx ON ocel_events(session_id);
CREATE INDEX IF NOT EXISTS ocel_events_activity_idx   ON ocel_events(activity);
CREATE INDEX IF NOT EXISTS ocel_events_timestamp_idx  ON ocel_events(timestamp_ms);

-- ── game_receipts ─────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS game_receipts (
  id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  -- NULL for Rust CLI cook receipts (no browser session)
  session_id       UUID REFERENCES game_sessions(id) ON DELETE SET NULL,
  -- 'PASS' | 'FAIL' | 'PENDING'
  verdict          TEXT NOT NULL DEFAULT 'PENDING',
  -- Human label: 'GameSessionProof', 'HTML5CookVerify', 'REAL-UE4-OCEL-001', etc.
  milestone        TEXT NOT NULL DEFAULT '',
  ocel_event_count INTEGER NOT NULL DEFAULT 0,
  -- Ordered unique activity names that were witnessed
  ocel_lifecycle   TEXT[] NOT NULL DEFAULT '{}',
  -- 'real_ue4' | 'rocket_cli' | 'synthetic' | 'unknown'
  engine_source    TEXT NOT NULL DEFAULT 'unknown',
  -- FNV-64 hex (cook) or sha256: hex (Playwright)
  receipt_hash     TEXT NOT NULL,
  proven_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
  -- wasm_mb, archive_dir, visual delta, etc.
  payload          JSONB NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS game_receipts_session_id_idx ON game_receipts(session_id);
CREATE INDEX IF NOT EXISTS game_receipts_verdict_idx    ON game_receipts(verdict);
CREATE INDEX IF NOT EXISTS game_receipts_proven_at_idx  ON game_receipts(proven_at DESC);
CREATE INDEX IF NOT EXISTS game_receipts_engine_src_idx ON game_receipts(engine_source);

-- ── leaderboard ───────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS leaderboard (
  id         BIGSERIAL PRIMARY KEY,
  player_id  UUID NOT NULL UNIQUE REFERENCES players(id) ON DELETE CASCADE,
  score      BIGINT NOT NULL DEFAULT 0,
  rank       INTEGER,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS leaderboard_score_idx ON leaderboard(score DESC);
