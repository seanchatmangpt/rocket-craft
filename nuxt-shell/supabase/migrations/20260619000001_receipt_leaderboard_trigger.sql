-- Leaderboard auto-update trigger: fires on every game_receipts INSERT.
--
-- When a PASS receipt arrives (from browser OR Rust CLI cook pipeline):
--   1. Find or create the leaderboard row for the session's player
--   2. Set score = max(current_score, ocel_event_count)
--   3. Update players.high_score
--   4. Recompute DENSE_RANK() for all players by score DESC
--
-- The trigger is SECURITY DEFINER so it can write leaderboard even under RLS.

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
  -- Only act on PASS verdicts
  IF NEW.verdict <> 'PASS' THEN
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

  -- Mark session closed with this receipt hash
  UPDATE game_sessions
     SET session_ended_at = now(),
         is_alive         = false,
         receipt_hash     = NEW.receipt_hash
   WHERE id = NEW.session_id
     AND session_ended_at IS NULL;

  RETURN NEW;
END;
$$;

-- Drop + recreate so re-running migrations is idempotent
DROP TRIGGER IF EXISTS on_receipt_pass ON game_receipts;

CREATE TRIGGER on_receipt_pass
  AFTER INSERT ON game_receipts
  FOR EACH ROW
  EXECUTE FUNCTION receipt_pass_leaderboard_trigger();
