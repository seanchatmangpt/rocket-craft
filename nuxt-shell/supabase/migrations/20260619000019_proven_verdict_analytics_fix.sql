-- Fix analytics views and daily rollup to count PROVEN alongside PASS.
--
-- receipt-finalize writes verdict='PROVEN' (chain-verified terminal state).
-- All views that counted verdict='PASS' were excluding PROVEN receipts from
-- pass rates, leaderboard stats, and the pipeline health snapshot.
-- This migration updates only the filter predicates; schema is unchanged.

-- ── daily_pipeline_rollup stored procedure (migration 000010) ─────────────────
-- Replace the pass_receipts COUNT filter to include PROVEN.

CREATE OR REPLACE FUNCTION refresh_daily_pipeline_rollup()
RETURNS void
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = public
AS $$
BEGIN
  INSERT INTO daily_pipeline_rollup (
    rollup_date,
    total_receipts,
    pass_receipts,
    fail_receipts,
    unique_sessions,
    unique_players,
    avg_ocel_events,
    pass_rate_pct
  )
  SELECT
    date_trunc('day', proven_at)::date              AS rollup_date,
    count(*)                                         AS total_receipts,
    count(*) FILTER (WHERE verdict IN ('PASS', 'PROVEN')) AS pass_receipts,
    count(*) FILTER (WHERE verdict = 'FAIL')         AS fail_receipts,
    count(DISTINCT session_id)                       AS unique_sessions,
    count(DISTINCT gs.player_id)                     AS unique_players,
    coalesce(avg(ocel_event_count), 0)              AS avg_ocel_events,
    CASE
      WHEN count(*) = 0 THEN NULL
      ELSE round(
        100.0 * count(*) FILTER (WHERE verdict IN ('PASS', 'PROVEN')) / count(*),
        1
      )
    END                                              AS pass_rate_pct
  FROM game_receipts gr
  LEFT JOIN game_sessions gs ON gs.id = gr.session_id
  WHERE proven_at >= now() - INTERVAL '1 day'
    AND proven_at IS NOT NULL
  GROUP BY 1
  ON CONFLICT (rollup_date) DO UPDATE SET
    total_receipts  = EXCLUDED.total_receipts,
    pass_receipts   = EXCLUDED.pass_receipts,
    fail_receipts   = EXCLUDED.fail_receipts,
    unique_sessions = EXCLUDED.unique_sessions,
    unique_players  = EXCLUDED.unique_players,
    avg_ocel_events = EXCLUDED.avg_ocel_events,
    pass_rate_pct   = EXCLUDED.pass_rate_pct;
END;
$$;

-- ── pipeline_health_snapshot view (migration 000004) ─────────────────────────
-- Recreate with PROVEN included in pass_count.

CREATE OR REPLACE VIEW pipeline_health_snapshot AS
SELECT
  count(*)                                                              AS total_receipts,
  count(*) FILTER (WHERE verdict IN ('PASS', 'PROVEN'))                AS pass_count,
  count(*) FILTER (WHERE verdict = 'FAIL')                             AS fail_count,
  count(*) FILTER (WHERE engine_source = 'real_ue4')                  AS real_ue4_sessions,
  CASE
    WHEN count(*) = 0 THEN NULL
    ELSE round(
      count(*) FILTER (WHERE verdict IN ('PASS', 'PROVEN'))::NUMERIC / count(*) * 100,
      1
    )
  END                                                                   AS pass_rate_pct,
  max(proven_at)                                                        AS last_proven_at
FROM game_receipts;

COMMENT ON VIEW pipeline_health_snapshot IS
  'Single-row pipeline health snapshot: receipt counts, PASS+PROVEN rate, '
  'real_ue4 sessions, last proven timestamp. Materialised manually by the '
  'daily rollup pg_cron job (migration 000010).';
