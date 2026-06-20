-- Daily observability rollups + pg_cron scheduled refresh
-- Pattern: dashboard.bak/supabase/migrations/007_views.sql (Evidence Center rollups)
-- Adapted for rocket-craft's game session + OCEL + receipt model

-- ── Materialized view: daily game session stats ────────────────────────────
CREATE MATERIALIZED VIEW IF NOT EXISTS session_stats_daily AS
SELECT
  date_trunc('day', session_started_at) AS session_date,
  count(*)                                AS total_sessions,
  count(*) FILTER (WHERE is_alive = false AND session_ended_at IS NOT NULL) AS closed_sessions,
  count(*) FILTER (WHERE is_alive = true)  AS live_sessions,
  avg(extract(epoch from (session_ended_at - session_started_at)))
    FILTER (WHERE session_ended_at IS NOT NULL) AS avg_session_duration_s,
  count(DISTINCT player_id) FILTER (WHERE player_id IS NOT NULL) AS unique_players
FROM game_sessions
GROUP BY date_trunc('day', session_started_at);

CREATE UNIQUE INDEX IF NOT EXISTS idx_session_stats_daily_date
  ON session_stats_daily (session_date DESC);

-- ── Materialized view: daily receipt stats ─────────────────────────────────
CREATE MATERIALIZED VIEW IF NOT EXISTS receipt_stats_daily AS
SELECT
  date_trunc('day', proven_at) AS receipt_date,
  count(*)                      AS total_receipts,
  count(*) FILTER (WHERE verdict = 'PASS')                 AS pass_receipts,
  count(*) FILTER (WHERE verdict = 'FAIL')                 AS fail_receipts,
  count(*) FILTER (WHERE engine_source = 'rocket_cli')     AS real_ue4_receipts,
  count(*) FILTER (WHERE engine_source = 'synthetic')      AS synthetic_rejected,
  avg(ocel_event_count)                                     AS avg_ocel_events,
  count(*) FILTER (WHERE ocel_event_count = 0)             AS zero_event_receipts
FROM game_receipts
WHERE proven_at IS NOT NULL
GROUP BY date_trunc('day', proven_at);

CREATE UNIQUE INDEX IF NOT EXISTS idx_receipt_stats_daily_date
  ON receipt_stats_daily (receipt_date DESC);

-- ── Materialized view: OCEL activity distribution (per day) ───────────────
CREATE MATERIALIZED VIEW IF NOT EXISTS ocel_activity_daily AS
SELECT
  date_trunc('day', timestamp_ms::numeric / 1000 * interval '1 second' + '1970-01-01'::timestamp) AS activity_date,
  activity,
  count(*)            AS event_count,
  count(DISTINCT session_id) AS unique_sessions
FROM ocel_events
GROUP BY 1, 2;

CREATE INDEX IF NOT EXISTS idx_ocel_activity_daily_date
  ON ocel_activity_daily (activity_date DESC, activity);

-- ── Live dashboard view (no materialize cost — derived from above) ─────────
CREATE OR REPLACE VIEW pipeline_dashboard AS
SELECT
  coalesce(r.receipt_date::text, s.session_date::text) AS day,
  coalesce(s.total_sessions, 0)       AS sessions,
  coalesce(s.unique_players, 0)       AS unique_players,
  coalesce(r.total_receipts, 0)       AS receipts,
  coalesce(r.pass_receipts, 0)        AS pass_receipts,
  coalesce(r.fail_receipts, 0)        AS fail_receipts,
  coalesce(r.real_ue4_receipts, 0)    AS real_ue4_receipts,
  coalesce(r.synthetic_rejected, 0)   AS synthetic_rejected,
  coalesce(r.avg_ocel_events, 0)      AS avg_ocel_events,
  CASE
    WHEN coalesce(r.total_receipts, 0) = 0 THEN NULL
    ELSE round(100.0 * r.pass_receipts / r.total_receipts, 1)
  END                                  AS pass_rate_pct
FROM session_stats_daily s
FULL OUTER JOIN receipt_stats_daily r ON r.receipt_date = s.session_date
ORDER BY day DESC;

-- ── pg_cron: refresh materialized views at midnight every day ─────────────
-- Graceful: only schedules if pg_cron extension is available.
DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'pg_cron') THEN
    -- Refresh session stats at 00:05 daily
    PERFORM cron.schedule(
      'rocket-session-stats-daily',
      '5 0 * * *',
      $cron$REFRESH MATERIALIZED VIEW CONCURRENTLY session_stats_daily;$cron$
    );
    -- Refresh receipt stats at 00:06 daily
    PERFORM cron.schedule(
      'rocket-receipt-stats-daily',
      '6 0 * * *',
      $cron$REFRESH MATERIALIZED VIEW CONCURRENTLY receipt_stats_daily;$cron$
    );
    -- Refresh OCEL activity at 00:07 daily
    PERFORM cron.schedule(
      'rocket-ocel-activity-daily',
      '7 0 * * *',
      $cron$REFRESH MATERIALIZED VIEW CONCURRENTLY ocel_activity_daily;$cron$
    );
    RAISE NOTICE 'pg_cron jobs scheduled: rocket-session-stats-daily, rocket-receipt-stats-daily, rocket-ocel-activity-daily';
  ELSE
    RAISE NOTICE 'pg_cron not available — daily rollups must be refreshed manually';
  END IF;
END;
$$;
