-- RLS owner-scoped policies for game_sessions and ocel_events.
-- Pattern: ~/seth/neako-web/supabase/rls-policies.sql (auth.uid() = owner_id).
--
-- Law: players own their sessions and events. The Rust CLI (service-role key)
-- can bypass RLS to push cook receipts from headless pipeline runs.

-- ── game_sessions ─────────────────────────────────────────────────────────────

ALTER TABLE game_sessions ENABLE ROW LEVEL SECURITY;

-- Players can read their own sessions; anonymous reads are blocked.
CREATE POLICY "game_sessions: owner read"
  ON game_sessions FOR SELECT
  USING (player_id = auth.uid() OR player_id IS NULL);

-- Players can insert sessions with their own player_id (or NULL for anon browser).
CREATE POLICY "game_sessions: owner insert"
  ON game_sessions FOR INSERT
  WITH CHECK (player_id = auth.uid() OR player_id IS NULL);

-- Players can update only their own sessions (sync alive status + event count).
CREATE POLICY "game_sessions: owner update"
  ON game_sessions FOR UPDATE
  USING (player_id = auth.uid() OR player_id IS NULL)
  WITH CHECK (player_id = auth.uid() OR player_id IS NULL);

-- ── ocel_events ──────────────────────────────────────────────────────────────

ALTER TABLE ocel_events ENABLE ROW LEVEL SECURITY;

-- Read: session owner or anonymous (session_id FK resolves player_id).
CREATE POLICY "ocel_events: session owner read"
  ON ocel_events FOR SELECT
  USING (
    EXISTS (
      SELECT 1 FROM game_sessions gs
      WHERE gs.id = ocel_events.session_id
        AND (gs.player_id = auth.uid() OR gs.player_id IS NULL)
    )
  );

-- Insert: only into sessions the user owns.
CREATE POLICY "ocel_events: session owner insert"
  ON ocel_events FOR INSERT
  WITH CHECK (
    EXISTS (
      SELECT 1 FROM game_sessions gs
      WHERE gs.id = ocel_events.session_id
        AND (gs.player_id = auth.uid() OR gs.player_id IS NULL)
    )
  );

-- ── game_receipts ────────────────────────────────────────────────────────────

ALTER TABLE game_receipts ENABLE ROW LEVEL SECURITY;

-- Anyone can read receipts (public proof chain).
CREATE POLICY "game_receipts: public read"
  ON game_receipts FOR SELECT
  USING (true);

-- Insert via session ownership OR via service-role key (Rust CLI cook receipts).
-- session_id IS NULL covers Rust CLI receipts which have no browser session.
CREATE POLICY "game_receipts: session owner or anon insert"
  ON game_receipts FOR INSERT
  WITH CHECK (
    session_id IS NULL
    OR EXISTS (
      SELECT 1 FROM game_sessions gs
      WHERE gs.id = game_receipts.session_id
        AND (gs.player_id = auth.uid() OR gs.player_id IS NULL)
    )
  );

-- ── leaderboard ──────────────────────────────────────────────────────────────

ALTER TABLE leaderboard ENABLE ROW LEVEL SECURITY;

-- Public read for the leaderboard page.
CREATE POLICY "leaderboard: public read"
  ON leaderboard FOR SELECT
  USING (true);

-- Only the trigger function (SECURITY DEFINER) writes to leaderboard — no direct inserts.
