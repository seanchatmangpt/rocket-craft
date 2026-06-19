-- Stale session detection and cleanup.
--
-- Invariant (from LieDetector pattern): a game_session row with is_alive=TRUE
-- must have had an ocel_event INSERT in the last 10 minutes.
-- If not, the session was never properly closed (e.g. browser crash, tab close).
--
-- This migration adds:
-- 1. close_stale_sessions() — closes sessions with no activity for >10 min
-- 2. A computed view: stale_sessions — sessions that are "alive" but quiet
-- 3. An index to support efficient stale detection
--
-- Run close_stale_sessions() via:
--   supabase functions invoke cleanup (or rocket supabase cleanup)
--   OR via pg_cron if available: SELECT cron.schedule('close-stale', '*/5 * * * *', 'SELECT close_stale_sessions()');

-- ── Index for stale detection ─────────────────────────────────────────────────

-- Fast lookup: find alive sessions started > 10 min ago with no recent events
CREATE INDEX IF NOT EXISTS idx_game_sessions_alive_started_v2
  ON game_sessions(session_started_at DESC)
  WHERE is_alive = true;

-- ── View: sessions that appear alive but have had no events recently ──────────

CREATE OR REPLACE VIEW stale_sessions AS
SELECT
  gs.id             AS session_id,
  gs.session_started_at,
  gs.ocel_event_count,
  gs.engine_source,
  gs.player_id,
  MAX(oe.timestamp_ms) AS last_event_ms,
  -- alive but no event in last 10 minutes
  CASE
    WHEN MAX(oe.timestamp_ms) IS NULL THEN TRUE                        -- no events at all
    WHEN MAX(oe.timestamp_ms) < (EXTRACT(EPOCH FROM NOW()) * 1000 - 600000) THEN TRUE
    ELSE FALSE
  END AS is_stale,
  NOW() - gs.session_started_at AS age
FROM game_sessions gs
LEFT JOIN ocel_events oe ON oe.session_id = gs.id
WHERE gs.is_alive = TRUE
GROUP BY gs.id, gs.session_started_at, gs.ocel_event_count, gs.engine_source, gs.player_id;

COMMENT ON VIEW stale_sessions IS
  'Game sessions that are marked is_alive=TRUE but have had no ocel_events '
  'in the last 10 minutes. Used by close_stale_sessions() to detect browser '
  'crashes and unclosed tabs. Invariant: alive sessions must have recent evidence.';

-- ── Function: close stale sessions ───────────────────────────────────────────

CREATE OR REPLACE FUNCTION close_stale_sessions(p_timeout_minutes INT DEFAULT 10)
RETURNS TABLE (closed_session_id UUID, age_minutes NUMERIC, event_count INT)
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = public
AS $$
BEGIN
  RETURN QUERY
  WITH stale AS (
    SELECT
      gs.id,
      EXTRACT(EPOCH FROM (NOW() - gs.session_started_at)) / 60 AS age_min,
      gs.ocel_event_count
    FROM game_sessions gs
    WHERE gs.is_alive = TRUE
      AND (
        -- No events at all, or last event was > timeout ago
        NOT EXISTS (
          SELECT 1 FROM ocel_events oe
          WHERE oe.session_id = gs.id
            AND oe.timestamp_ms > (EXTRACT(EPOCH FROM NOW()) * 1000 - p_timeout_minutes * 60000)
        )
      )
      -- Only close sessions older than the timeout threshold
      AND gs.session_started_at < NOW() - (p_timeout_minutes || ' minutes')::INTERVAL
  ),
  closed AS (
    UPDATE game_sessions
    SET
      is_alive = FALSE,
      session_ended_at = NOW(),
      -- Tag as stale-closed in metadata for audit trail
      metadata = COALESCE(metadata, '{}'::jsonb) || jsonb_build_object(
        'closed_by', 'close_stale_sessions',
        'closed_at', NOW()::TEXT,
        'timeout_minutes', p_timeout_minutes
      )
    FROM stale
    WHERE game_sessions.id = stale.id
    RETURNING game_sessions.id, stale.age_min, stale.ocel_event_count
  )
  SELECT closed.id, ROUND(closed.age_min::NUMERIC, 1), closed.ocel_event_count
  FROM closed;
END;
$$;

COMMENT ON FUNCTION close_stale_sessions(INT) IS
  'Closes game_sessions rows that are is_alive=TRUE but have had no ocel_events '
  'in the last p_timeout_minutes minutes. Tags the metadata with closed_by='
  'close_stale_sessions for the audit trail. '
  'Returns the closed session IDs and their ages. '
  'Call via: SELECT * FROM close_stale_sessions(10);';
