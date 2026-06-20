-- Fix: leaderboard trigger was checking `verdict = 'PASS'` only.
--
-- receipt-finalize writes `verdict = 'PROVEN'` (chain-verified terminal state);
-- session-seed and cook-receipt write `verdict = 'PASS'` (unverified or cook receipt).
-- Both are legitimate proof events that should rank the player.
--
-- The headless gameplay loop goes: session-seed (PASS) → receipt-finalize (PROVEN).
-- Before this fix, only the session-seed receipt triggered the leaderboard, and
-- only if player_id was set on that seed — the finalize PROVEN update was invisible
-- to the trigger.
--
-- Fix: change the early-exit guard to accept both PASS and PROVEN.

CREATE OR REPLACE FUNCTION receipt_pass_leaderboard_trigger()
RETURNS TRIGGER
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = public
AS $$
DECLARE
  v_player_id  UUID;
  v_new_score  BIGINT;
BEGIN
  -- Act on PASS (seed/cook receipts) AND PROVEN (chain-verified receipt-finalize)
  IF NEW.verdict NOT IN ('PASS', 'PROVEN') THEN
    RETURN NEW;
  END IF;

  -- Resolve player_id from session (may be NULL for CLI cook receipts)
  IF NEW.session_id IS NOT NULL THEN
    SELECT player_id INTO v_player_id
      FROM game_sessions
     WHERE id = NEW.session_id;
  END IF;

  -- No player to rank — still record the receipt, just skip leaderboard update
  IF v_player_id IS NULL THEN
    RETURN NEW;
  END IF;

  -- Score = OCEL event count (proxy for session depth / engagement)
  v_new_score := COALESCE(NEW.ocel_event_count, 0);

  -- Upsert leaderboard row — only raise score, never lower it
  INSERT INTO leaderboard (player_id, score, updated_at)
       VALUES (v_player_id, v_new_score, now())
  ON CONFLICT (player_id) DO UPDATE
     SET score      = GREATEST(leaderboard.score, EXCLUDED.score),
         updated_at = now()
   WHERE EXCLUDED.score > leaderboard.score;

  -- Sync players.high_score
  UPDATE players
     SET high_score = GREATEST(high_score, v_new_score)
   WHERE id = v_player_id;

  -- Recompute ranks across all players using DENSE_RANK
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
