-- Owner-scoped RLS for game_sessions.
--
-- Gap: game_sessions is currently world-readable via anon key. Session metadata
-- (player_id, receipt_hash, engine_source) is PII-adjacent and should only be
-- visible to the service role or the owning player.
--
-- Policy hierarchy (evaluated in order by Postgres):
--   1. service_role_all   — full CRUD bypass (server-side API calls)
--   2. player_read_own    — authenticated users can SELECT their own rows
--   3. deny_anon_select   — anonymous callers get nothing
--
-- Pattern mirrors migration 20260619000005 (players RLS) — enable RLS, then
-- layer policies from most-permissive (service role) to least (anon deny).

ALTER TABLE game_sessions ENABLE ROW LEVEL SECURITY;

-- ── Service role: unrestricted (server-side receipt-finalize, session-start, cleanup) ──

DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_policies
    WHERE schemaname = 'public'
      AND tablename  = 'game_sessions'
      AND policyname = 'service_role_all'
  ) THEN
    EXECUTE $policy$
      CREATE POLICY "service_role_all"
        ON game_sessions
        FOR ALL
        TO service_role
        USING (true)
        WITH CHECK (true)
    $policy$;
  END IF;
END;
$$;

-- ── Authenticated players: read their own session rows only ──

DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_policies
    WHERE schemaname = 'public'
      AND tablename  = 'game_sessions'
      AND policyname = 'player_read_own'
  ) THEN
    EXECUTE $policy$
      CREATE POLICY "player_read_own"
        ON game_sessions
        FOR SELECT
        TO authenticated
        USING (player_id = auth.uid()::text)
    $policy$;
  END IF;
END;
$$;

-- ── Anon: explicitly denied (no implicit access even if RLS is bypassed elsewhere) ──
--
-- A CREATE POLICY … USING (false) is the canonical Supabase pattern for explicit
-- deny. This prevents accidental exposure if the anon key is used in a service
-- context that hasn't switched to the service role.

DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_policies
    WHERE schemaname = 'public'
      AND tablename  = 'game_sessions'
      AND policyname = 'deny_anon_select'
  ) THEN
    EXECUTE $policy$
      CREATE POLICY "deny_anon_select"
        ON game_sessions
        FOR SELECT
        TO anon
        USING (false)
    $policy$;
  END IF;
END;
$$;
