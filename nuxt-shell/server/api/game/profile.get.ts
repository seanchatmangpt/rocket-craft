/**
 * GET /api/game/profile?player_id=<uuid>
 *
 * Server-side player profile — consolidates 3 client-side Supabase queries
 * (players table, game_sessions history, leaderboard rank) into one cacheable
 * server call. The service role key never leaks to the browser.
 *
 * Returns:
 *   {
 *     player: { id, username, high_score, created_at } | null,
 *     rank: number | null,
 *     sessions: SessionSummary[],
 *     totals: { total_events, sessions_with_proof },
 *   }
 *
 * Pattern: ~/dashboard.bak/server/api/customers.ts
 */

import { createClient } from '@supabase/supabase-js';

export default defineEventHandler(async (event) => {
  const query = getQuery(event);
  const playerId = typeof query.player_id === 'string' ? query.player_id.trim() : null;

  if (!playerId) {
    throw createError({ statusCode: 400, statusMessage: 'player_id query param required' });
  }

  const config = useRuntimeConfig(event);
  const supabaseUrl = (config.public.supabaseUrl as string) || 'http://localhost:54321';
  const serviceKey = (config.supabaseServiceRoleKey as string) || (config.public.supabaseAnonKey as string);

  if (!supabaseUrl || !serviceKey) {
    throw createError({ statusCode: 503, statusMessage: 'Supabase not configured' });
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const sb = createClient<any>(supabaseUrl, serviceKey);

  const [playerRes, sessionsRes, rankRes] = await Promise.all([
    sb.from('players')
      .select('id, username, high_score, created_at')
      .eq('auth_user_id', playerId)
      .single(),
    sb.from('game_sessions')
      .select('id, is_alive, ocel_event_count, engine_source, session_started_at, session_ended_at')
      .eq('player_id', playerId)
      .order('session_started_at', { ascending: false })
      .limit(20),
    sb.from('leaderboard')
      .select('rank')
      .eq('player_id', playerId)
      .single(),
  ]);

  const sessions = (sessionsRes.data ?? []) as Array<{
    id: string;
    is_alive: boolean;
    ocel_event_count: number;
    engine_source: string;
    session_started_at: string;
    session_ended_at: string | null;
  }>;

  const totalEvents = sessions.reduce((sum, s) => sum + (s.ocel_event_count ?? 0), 0);
  const sessionsWithProof = sessions.filter(s => (s.ocel_event_count ?? 0) > 0).length;

  return {
    player: playerRes.data ?? null,
    rank: rankRes.data?.rank ?? null,
    sessions,
    totals: {
      total_events: totalEvents,
      sessions_with_proof: sessionsWithProof,
    },
  };
});
