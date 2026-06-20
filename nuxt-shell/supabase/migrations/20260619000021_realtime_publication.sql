-- Enable Supabase Realtime (postgres_changes) for the tables the browser
-- subscribes to. Without membership in the supabase_realtime publication, a
-- channel .on('postgres_changes', { table: 'leaderboard' }) subscribes OK but
-- NEVER receives events — the live leaderboard would silently never update.
--
-- useRocketRealtimeLeaderboard subscribes to `leaderboard` (cross-client rank
-- updates). useRocketSessionRealtime subscribes to `game_sessions`.

DO $$
BEGIN
  -- Create the publication if the local stack didn't already (hosted Supabase has it).
  IF NOT EXISTS (SELECT 1 FROM pg_publication WHERE pubname = 'supabase_realtime') THEN
    CREATE PUBLICATION supabase_realtime;
  END IF;

  -- Add leaderboard if not already a member.
  IF NOT EXISTS (
    SELECT 1 FROM pg_publication_tables
    WHERE pubname = 'supabase_realtime' AND schemaname = 'public' AND tablename = 'leaderboard'
  ) THEN
    ALTER PUBLICATION supabase_realtime ADD TABLE public.leaderboard;
  END IF;

  -- Add game_sessions if not already a member.
  IF NOT EXISTS (
    SELECT 1 FROM pg_publication_tables
    WHERE pubname = 'supabase_realtime' AND schemaname = 'public' AND tablename = 'game_sessions'
  ) THEN
    ALTER PUBLICATION supabase_realtime ADD TABLE public.game_sessions;
  END IF;
END;
$$;
