-- Migration: pipeline analytics views
-- Adapted for rocket-craft tables: game_receipts, game_sessions, ocel_events, players, leaderboard.
-- Uses regular views only — no materialized views (pg_cron not required).

-- ── receipt_stats_daily ───────────────────────────────────────────────────────
-- Count of receipts per calendar day, broken down by verdict and engine_source.
-- Useful for trending PASS/FAIL rates and tracking which engine sources are active.

CREATE OR REPLACE VIEW receipt_stats_daily AS
SELECT
  date_trunc('day', proven_at)::DATE  AS receipt_date,
  verdict,
  engine_source,
  count(*)                             AS receipt_count,
  -- Convenience: hash-chain coverage
  count(*) FILTER (WHERE receipt_hash IS NOT NULL AND receipt_hash <> '') AS receipts_with_hash
FROM game_receipts
GROUP BY
  date_trunc('day', proven_at)::DATE,
  verdict,
  engine_source
ORDER BY receipt_date DESC, verdict, engine_source;

COMMENT ON VIEW receipt_stats_daily IS
  'Daily receipt counts grouped by verdict and engine_source. '
  'Use to trend PASS/FAIL rates over time without materialized views.';

-- ── session_lifecycle_summary ─────────────────────────────────────────────────
-- Per-session summary: event count, distinct activities, first/last timestamp.
-- Provides the process-mining input needed to verify activity ordering.

CREATE OR REPLACE VIEW session_lifecycle_summary AS
SELECT
  gs.id                                                   AS session_id,
  gs.session_started_at,
  gs.session_ended_at,
  gs.is_alive,
  count(oe.id)                                            AS event_count,
  -- Ordered distinct activity names witnessed in this session
  array_agg(DISTINCT oe.activity ORDER BY oe.activity)    AS distinct_activities,
  min(oe.timestamp_ms)                                    AS first_event_ms,
  max(oe.timestamp_ms)                                    AS last_event_ms,
  -- Duration in milliseconds (NULL when session still alive or no events)
  CASE
    WHEN count(oe.id) > 0 THEN max(oe.timestamp_ms) - min(oe.timestamp_ms)
    ELSE NULL
  END                                                     AS duration_ms,
  -- Latest receipt verdict for this session (NULL if no receipt yet)
  (
    SELECT gr.verdict
    FROM   game_receipts gr
    WHERE  gr.session_id = gs.id
    ORDER  BY gr.proven_at DESC
    LIMIT  1
  )                                                       AS latest_verdict
FROM      game_sessions gs
LEFT JOIN ocel_events   oe ON oe.session_id = gs.id
GROUP BY  gs.id, gs.session_started_at, gs.session_ended_at, gs.is_alive
ORDER BY  gs.session_started_at DESC;

COMMENT ON VIEW session_lifecycle_summary IS
  'Per-session event count, distinct OCEL activities, and timestamp range. '
  'Use as the process-mining input to verify lawful activity ordering per session.';

-- ── pipeline_health ───────────────────────────────────────────────────────────
-- Single-row overall health snapshot: total receipts, PASS rate, real_ue4 sessions.
-- Intentionally a plain SELECT so it always reflects live data.

CREATE OR REPLACE VIEW pipeline_health AS
SELECT
  -- Receipt totals
  count(*)                                                        AS total_receipts,
  count(*) FILTER (WHERE verdict = 'PASS')                        AS pass_count,
  count(*) FILTER (WHERE verdict = 'FAIL')                        AS fail_count,
  count(*) FILTER (WHERE verdict = 'PENDING')                     AS pending_count,

  -- PASS rate (NULL when no receipts)
  CASE
    WHEN count(*) > 0
    THEN round(
      count(*) FILTER (WHERE verdict = 'PASS')::NUMERIC / count(*) * 100,
      2
    )
    ELSE NULL
  END                                                             AS pass_rate_pct,

  -- Engine source breakdown
  count(*) FILTER (WHERE engine_source = 'real_ue4')             AS real_ue4_receipts,
  count(*) FILTER (WHERE engine_source = 'rocket_cli')           AS rocket_cli_receipts,
  count(*) FILTER (WHERE engine_source = 'synthetic')            AS synthetic_receipts,
  count(*) FILTER (WHERE engine_source = 'unknown')              AS unknown_receipts,

  -- Sessions that have at least one real_ue4 receipt
  (
    SELECT count(DISTINCT gr.session_id)
    FROM   game_receipts gr
    WHERE  gr.engine_source = 'real_ue4'
      AND  gr.session_id IS NOT NULL
  )                                                               AS sessions_with_real_ue4,

  -- Total sessions
  (SELECT count(*) FROM game_sessions)                           AS total_sessions,

  -- Active sessions
  (SELECT count(*) FROM game_sessions WHERE is_alive = TRUE)     AS active_sessions,

  -- Total players
  (SELECT count(*) FROM players)                                 AS total_players,

  -- Timestamp of last proven receipt
  max(proven_at)                                                  AS last_receipt_at

FROM game_receipts;

COMMENT ON VIEW pipeline_health IS
  'Single-row pipeline health snapshot: receipt counts, PASS rate, '
  'real_ue4 session count, player count. Reflects live data on every query.';
