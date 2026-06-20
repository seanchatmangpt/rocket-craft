-- Seed data for local development.
-- Applied automatically by `supabase start` and `supabase db reset`.
-- Safe to re-run (all inserts are ON CONFLICT DO NOTHING).

-- Dev pilot: maps to the local anon auth session
INSERT INTO players (id, username, high_score)
VALUES ('00000000-0000-0000-0000-000000000001', 'dev-pilot', 0)
ON CONFLICT (id) DO NOTHING;

-- Seed one completed session so receipts and leaderboard pages aren't blank
INSERT INTO game_sessions (
  id, player_id, session_started_at, session_ended_at,
  is_alive, ocel_event_count, engine_source, receipt_hash
) VALUES (
  '00000000-0000-0000-0000-000000000002',
  '00000000-0000-0000-0000-000000000001',
  now() - interval '1 hour',
  now() - interval '55 minutes',
  false, 7, 'synthetic', 'seed-receipt-hash'
) ON CONFLICT (id) DO NOTHING;

-- Seed a PASS receipt (will fire the trigger to populate leaderboard)
INSERT INTO game_receipts (
  id, session_id, verdict, milestone,
  ocel_event_count, ocel_lifecycle, engine_source,
  receipt_hash, proven_at, payload
) VALUES (
  '00000000-0000-0000-0000-000000000003',
  '00000000-0000-0000-0000-000000000002',
  'PASS',
  'SeedSession',
  7,
  ARRAY['GameSessionStarted', 'FrameRendered', 'InputAdmitted'],
  'synthetic',
  'seed-receipt-hash',
  now() - interval '55 minutes',
  '{"note": "seed data for local dev"}'::jsonb
) ON CONFLICT (id) DO NOTHING;
