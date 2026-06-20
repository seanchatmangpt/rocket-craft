-- Add idempotency_key to game_sessions so session-seed retries return the
-- same session instead of creating duplicates.
-- A unique constraint ensures exactly-once semantics per key.

alter table game_sessions
  add column if not exists idempotency_key text unique;

-- Fast lookup by key (used on every seeded request that passes a key)
create index if not exists idx_game_sessions_idempotency_key
  on game_sessions (idempotency_key)
  where idempotency_key is not null;
