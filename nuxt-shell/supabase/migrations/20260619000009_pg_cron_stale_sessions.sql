-- Migration 000009: pg_cron schedule for automatic stale session cleanup
--
-- Runs close_stale_sessions() every 5 minutes via pg_cron.
-- safe to run multiple times (uses IF NOT EXISTS / DO $$ guards).
--
-- Requires: pg_cron extension enabled in Supabase project settings.
-- Local dev: pg_cron is NOT available in supabase start (local Postgres).
-- This migration is a no-op locally and takes effect only in hosted Supabase.

-- Enable pg_cron if available (hosted Supabase projects have it; local dev doesn't)
DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM pg_available_extensions WHERE name = 'pg_cron') THEN
    CREATE EXTENSION IF NOT EXISTS pg_cron;
    RAISE NOTICE 'pg_cron enabled';
  ELSE
    RAISE NOTICE 'pg_cron not available (local dev) — stale session cleanup must be triggered manually';
  END IF;
END;
$$;

-- Schedule cleanup every 5 minutes (only if pg_cron is installed)
DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'pg_cron') THEN
    -- Remove any existing job with this name before re-creating (idempotent)
    PERFORM cron.unschedule('rocket-stale-sessions')
    WHERE EXISTS (SELECT 1 FROM cron.job WHERE jobname = 'rocket-stale-sessions');

    PERFORM cron.schedule(
      'rocket-stale-sessions',
      '*/5 * * * *',
      $$SELECT close_stale_sessions()$$
    );
    RAISE NOTICE 'pg_cron job rocket-stale-sessions scheduled (every 5 min)';
  END IF;
END;
$$;

-- View: show which sessions cron has closed (for observability)
CREATE OR REPLACE VIEW pg_cron_session_audit AS
SELECT
  gs.id,
  gs.project_name,
  gs.session_started_at,
  gs.session_ended_at,
  gs.is_alive,
  gs.engine_source,
  EXTRACT(EPOCH FROM (gs.session_ended_at - gs.session_started_at)) AS duration_secs
FROM game_sessions gs
WHERE gs.is_alive = false
  AND gs.session_ended_at IS NOT NULL
ORDER BY gs.session_ended_at DESC;

COMMENT ON VIEW pg_cron_session_audit IS
  'Sessions closed by rocket-stale-sessions pg_cron job or manual close_stale_sessions() call';
