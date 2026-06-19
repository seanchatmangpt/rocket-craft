-- Migration: add output_hash to game_receipts
-- output_hash = SHA-256 hex of the cooked WASM artifact bytes.
-- Written by the Rust CLI (rocket html5 verify → as_supabase_receipt).
-- Used by verify_html5_pipeline.sh [6/5] cook-to-game cross-check.
-- Browser receipts will have NULL (no WASM artifact on the client side).

ALTER TABLE game_receipts
  ADD COLUMN IF NOT EXISTS output_hash text;

-- Index for the cross-check query: find all receipts for a given WASM binary.
CREATE INDEX IF NOT EXISTS idx_game_receipts_output_hash
  ON game_receipts (output_hash)
  WHERE output_hash IS NOT NULL;

-- Comment for schema introspection
COMMENT ON COLUMN game_receipts.output_hash IS
  'SHA-256 hex of the cooked WASM artifact (64 chars). NULL for browser-only receipts.';
