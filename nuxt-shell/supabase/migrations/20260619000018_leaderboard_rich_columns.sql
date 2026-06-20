-- Add rich stat columns to leaderboard that leaderboard.get.ts selects.
--
-- Gap: base leaderboard table (migration 000000) only has (player_id, score, rank,
-- updated_at). The endpoint queries display_name, total_receipts, pass_receipts,
-- fail_receipts, pass_rate_pct, last_pass_at, best_ocel_events — all missing.
-- Without them, Supabase returns null for every stat column and the leaderboard
-- appears empty to the client even when sessions are proven.

ALTER TABLE leaderboard
  ADD COLUMN IF NOT EXISTS display_name      TEXT,
  ADD COLUMN IF NOT EXISTS total_receipts    BIGINT  NOT NULL DEFAULT 0,
  ADD COLUMN IF NOT EXISTS pass_receipts     BIGINT  NOT NULL DEFAULT 0,
  ADD COLUMN IF NOT EXISTS fail_receipts     BIGINT  NOT NULL DEFAULT 0,
  ADD COLUMN IF NOT EXISTS pass_rate_pct     NUMERIC(5,2),
  ADD COLUMN IF NOT EXISTS last_pass_at      TIMESTAMPTZ,
  ADD COLUMN IF NOT EXISTS best_ocel_events  BIGINT;

COMMENT ON COLUMN leaderboard.display_name     IS 'Copied from players.display_name at upsert time — denormalised for O(1) leaderboard reads without a join.';
COMMENT ON COLUMN leaderboard.total_receipts   IS 'Cumulative receipt count for this player (PASS + FAIL).';
COMMENT ON COLUMN leaderboard.pass_receipts    IS 'Cumulative PASS/PROVEN receipts.';
COMMENT ON COLUMN leaderboard.fail_receipts    IS 'Cumulative FAIL receipts.';
COMMENT ON COLUMN leaderboard.pass_rate_pct    IS 'pass_receipts / total_receipts * 100, 2 dp.';
COMMENT ON COLUMN leaderboard.last_pass_at     IS 'Timestamp of the most recent PASS or PROVEN receipt.';
COMMENT ON COLUMN leaderboard.best_ocel_events IS 'Highest ocel_event_count across all PASS/PROVEN receipts — indicates deepest proven session.';

-- Update the trigger function to maintain the new columns.
-- This supersedes migration 000017 (which only fixed the PROVEN verdict guard).

CREATE OR REPLACE FUNCTION receipt_pass_leaderboard_trigger()
RETURNS TRIGGER
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = public
AS $$
DECLARE
  v_player_id      UUID;
  v_display_name   TEXT;
  v_new_score      BIGINT;
  v_ocel           BIGINT;
BEGIN
  -- Act on PASS (seed/cook receipts) AND PROVEN (chain-verified receipt-finalize)
  IF NEW.verdict NOT IN ('PASS', 'PROVEN') THEN
    RETURN NEW;
  END IF;

  -- Resolve player_id from session (may be NULL for CLI cook receipts)
  IF NEW.session_id IS NOT NULL THEN
    SELECT gs.player_id, p.display_name
      INTO v_player_id, v_display_name
      FROM game_sessions gs
      LEFT JOIN players p ON p.id = gs.player_id
     WHERE gs.id = NEW.session_id;
  END IF;

  -- No player — still record receipt, skip leaderboard
  IF v_player_id IS NULL THEN
    RETURN NEW;
  END IF;

  v_ocel      := COALESCE(NEW.ocel_event_count, 0);
  v_new_score := v_ocel;

  -- Upsert leaderboard row with full stat update
  INSERT INTO leaderboard (
    player_id, display_name, score,
    total_receipts, pass_receipts, fail_receipts,
    pass_rate_pct, last_pass_at, best_ocel_events,
    updated_at
  ) VALUES (
    v_player_id, v_display_name, v_new_score,
    1, 1, 0,
    100.00, NEW.proven_at,
    v_ocel,
    now()
  )
  ON CONFLICT (player_id) DO UPDATE SET
    display_name     = COALESCE(EXCLUDED.display_name, leaderboard.display_name),
    score            = GREATEST(leaderboard.score, EXCLUDED.score),
    total_receipts   = leaderboard.total_receipts + 1,
    pass_receipts    = leaderboard.pass_receipts + 1,
    pass_rate_pct    = round(
                         100.0 * (leaderboard.pass_receipts + 1) /
                         (leaderboard.total_receipts + 1),
                         2
                       ),
    last_pass_at     = GREATEST(leaderboard.last_pass_at, NEW.proven_at),
    best_ocel_events = GREATEST(leaderboard.best_ocel_events, EXCLUDED.best_ocel_events),
    updated_at       = now();

  -- Sync players.high_score
  UPDATE players
     SET high_score = GREATEST(high_score, v_new_score)
   WHERE id = v_player_id;

  -- Recompute DENSE_RANK for all players
  UPDATE leaderboard lb
     SET rank = ranked.new_rank
    FROM (
           SELECT player_id,
                  DENSE_RANK() OVER (ORDER BY score DESC) AS new_rank
             FROM leaderboard
         ) ranked
   WHERE lb.player_id = ranked.player_id;

  RETURN NEW;
END;
$$;
