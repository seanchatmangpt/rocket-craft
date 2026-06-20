-- The leaderboard trigger (migration 000001) only fires on INSERT into game_receipts.
--
-- receipt-finalize performs an UPDATE: it changes verdict from 'PASS' → 'PROVEN'
-- on an existing row. This UPDATE was invisible to the trigger, so the leaderboard
-- was never updated from the headless loop finalize step.
--
-- Fix: add a second trigger that fires AFTER UPDATE on game_receipts, guarded so
-- it only fires when verdict transitions TO 'PROVEN' (not on every update).

DROP TRIGGER IF EXISTS on_receipt_proven ON game_receipts;

CREATE TRIGGER on_receipt_proven
  AFTER UPDATE OF verdict ON game_receipts
  FOR EACH ROW
  WHEN (NEW.verdict = 'PROVEN' AND OLD.verdict IS DISTINCT FROM 'PROVEN')
  EXECUTE FUNCTION receipt_pass_leaderboard_trigger();
