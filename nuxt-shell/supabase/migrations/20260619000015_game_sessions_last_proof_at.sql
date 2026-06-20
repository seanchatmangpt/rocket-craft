-- Add last_proof_at column to game_sessions.
--
-- Gap: game_sessions has no dedicated timestamp for "when was this session proven".
-- receipt-finalize writes session_ended_at and receipt_hash when verdict === 'PROVEN',
-- but that conflates the session-end time with the proof time (they should be the
-- same for a normal flow, but diverge when a session ends without being proven, or
-- when proof arrives after a delayed finalize call).
--
-- last_proof_at is written by receipt-finalize alongside receipt_hash and
-- session_ended_at. It lets the stale-session cleanup (migration 20260619000008)
-- query "sessions that ended but have never been proven" efficiently:
--
--   WHERE session_ended_at < now() - INTERVAL '1 hour'
--     AND last_proof_at IS NULL
--
-- Without this column that query requires a LEFT JOIN on game_receipts, which is
-- O(sessions × receipts) and blocks the cleanup job on large tables.

ALTER TABLE game_sessions
  ADD COLUMN IF NOT EXISTS last_proof_at TIMESTAMPTZ;

COMMENT ON COLUMN game_sessions.last_proof_at IS
  'Timestamp set by receipt-finalize when the session reaches verdict=PROVEN. '
  'NULL means the session has not yet been proven (may still be active or stalled). '
  'Used by the stale-session cleanup job to detect sessions that ended without proof.';

-- Partial index: only rows that have been proven are interesting for proof-time
-- queries. The WHERE clause keeps the index small (NULL rows excluded).
CREATE INDEX IF NOT EXISTS idx_game_sessions_last_proof_at
  ON game_sessions (last_proof_at)
  WHERE last_proof_at IS NOT NULL;
