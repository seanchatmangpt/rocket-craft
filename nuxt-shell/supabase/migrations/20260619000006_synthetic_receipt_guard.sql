-- Database-level guard: reject synthetic engine_source values at the constraint layer.
-- Pattern: defense-in-depth — the server route also rejects these, but the DB constraint
-- makes it impossible even via direct SQL or psql inserts (e.g., from seed scripts or
-- manual data entry). Matches the Poka-Yoke principle: error prevention, not detection.
--
-- Only receipts where engine_source is NULL, 'real_ue4', or 'unknown' are admitted.
-- 'unknown' is allowed (browser game without UE4 loaded) but will never produce PASS
-- because verifyLifecycle() requires FrameRendered which only fires when UE4 is live.

ALTER TABLE game_receipts
  ADD CONSTRAINT chk_no_synthetic_engine_source
  CHECK (
    engine_source IS NULL
    OR engine_source NOT IN ('synthetic', 'sim', 'fake', 'stub', 'mock')
  );

-- Mirror the same constraint on game_sessions (sessions should also record only
-- real or unknown engine sources; synthetic sessions pollute the lifecycle analytics).
ALTER TABLE game_sessions
  ADD CONSTRAINT chk_sessions_no_synthetic_engine_source
  CHECK (
    engine_source IS NULL
    OR engine_source NOT IN ('synthetic', 'sim', 'fake', 'stub', 'mock')
  );

COMMENT ON CONSTRAINT chk_no_synthetic_engine_source ON game_receipts IS
  'Reject synthetic receipts at the DB layer. '
  'Van der Aalst doctrine: only real evidence may enter the event log. '
  'The server route (receipt.post.ts) also rejects these — DB constraint is belt-and-suspenders.';
