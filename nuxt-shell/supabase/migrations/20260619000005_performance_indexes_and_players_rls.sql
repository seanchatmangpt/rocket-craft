-- Performance indexes for the OCEL pipeline.
-- Pattern: ~/dashboard.bak/supabase/migrations/006_indexes.sql (hash-chain query patterns)
--
-- Critical indexes for verify_event_chain performance:
--   idx_ocel_events_hash     — integrity verification (JOIN on event_hash)
--   idx_ocel_events_prev_hash — chain traversal (find next link)
-- Both are used by the verify_event_chain() RPC inner loop.

-- ── ocel_events: hash chain lookup indexes ────────────────────────────────────

-- Fast chain walk: the RPC orders by (session_id, seq) — already covered by
-- the primary key + session_id index, but this composite speeds up the LOOP.
CREATE INDEX IF NOT EXISTS idx_ocel_events_session_seq
  ON ocel_events(session_id, seq ASC);

-- Hash integrity: fast lookup by event_hash (for cross-session dedup and proof gen)
CREATE INDEX IF NOT EXISTS idx_ocel_events_hash
  ON ocel_events(event_hash);

-- Chain break detection: find events whose prev_hash references a specific hash
CREATE INDEX IF NOT EXISTS idx_ocel_events_prev_hash
  ON ocel_events(prev_hash) WHERE prev_hash IS NOT NULL;

-- Activity filtering: efficient process discovery queries (find all FrameRendered, etc.)
CREATE INDEX IF NOT EXISTS idx_ocel_events_activity_ts
  ON ocel_events(session_id, activity, timestamp_ms DESC);

-- JSONB attribute search (for pm4py attribute filter queries)
CREATE INDEX IF NOT EXISTS idx_ocel_events_attributes_gin
  ON ocel_events USING gin(attributes);

-- ── game_receipts: verdict + engine_source filtering ─────────────────────────

-- Pipeline health view queries: verdict=PASS filter with proven_at ordering
CREATE INDEX IF NOT EXISTS idx_game_receipts_verdict_proven
  ON game_receipts(verdict, proven_at DESC);

-- Engine source filtering: `WHERE engine_source = 'real_ue4'`
CREATE INDEX IF NOT EXISTS idx_game_receipts_engine_proven
  ON game_receipts(engine_source, proven_at DESC);

-- ── game_sessions: lifecycle queries ─────────────────────────────────────────

-- Find active sessions quickly (pipeline_health view uses this)
CREATE INDEX IF NOT EXISTS idx_game_sessions_alive_started
  ON game_sessions(is_alive, session_started_at DESC) WHERE is_alive = true;

-- Engine source on sessions (for session_lifecycle_summary view)
CREATE INDEX IF NOT EXISTS idx_game_sessions_engine_source
  ON game_sessions(engine_source, session_started_at DESC);

-- ── players: RLS + player lookup ─────────────────────────────────────────────

ALTER TABLE players ENABLE ROW LEVEL SECURITY;

-- Anyone can read player profiles (leaderboard is public)
CREATE POLICY "players: public read"
  ON players FOR SELECT
  USING (true);

-- Players can insert their own profile row (auth_user_id must match)
CREATE POLICY "players: self insert"
  ON players FOR INSERT
  WITH CHECK (auth_user_id = auth.uid() OR auth_user_id IS NULL);

-- Players can update their own profile
CREATE POLICY "players: self update"
  ON players FOR UPDATE
  USING (auth_user_id = auth.uid() OR auth_user_id IS NULL)
  WITH CHECK (auth_user_id = auth.uid() OR auth_user_id IS NULL);

-- ── Utility function: upsert player profile from auth session ─────────────────
-- Called by the profile page on first login to create the players row.

CREATE OR REPLACE FUNCTION upsert_player_profile(
  p_username TEXT DEFAULT NULL
)
RETURNS UUID
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = public
AS $$
DECLARE
  v_user_id UUID;
  v_player_id UUID;
BEGIN
  v_user_id := auth.uid();
  IF v_user_id IS NULL THEN
    RAISE EXCEPTION 'not authenticated';
  END IF;

  INSERT INTO players (auth_user_id, username, high_score)
  VALUES (v_user_id, p_username, 0)
  ON CONFLICT (auth_user_id) DO UPDATE
    SET username = COALESCE(EXCLUDED.username, players.username)
  RETURNING id INTO v_player_id;

  RETURN v_player_id;
END;
$$;

COMMENT ON FUNCTION upsert_player_profile(TEXT) IS
  'Create or update the authenticated user''s player profile. '
  'Returns the players.id UUID. '
  'Call from the profile page after auth.getSession() succeeds.';
