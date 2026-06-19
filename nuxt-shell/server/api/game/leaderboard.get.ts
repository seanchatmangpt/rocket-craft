/**
 * GET /api/game/leaderboard
 *
 * Returns the top-N ranked players from the leaderboard table.
 * The leaderboard is maintained automatically by the Postgres trigger
 * on_receipt_pass (migration 000001) — no manual writes needed.
 *
 * Query params:
 *   limit  — max rows (default 20, max 100)
 *   offset — pagination offset (default 0)
 *
 * Returns:
 *   { rows: LeaderboardRow[], total: number, cached_at: string }
 *
 * Pattern: ~/dashboard.bak/server/api/customers.ts (ranked list with Nitro KV cache)
 * Server-side so the service role key never leaks to the browser.
 */

import { createClient } from '@supabase/supabase-js';

interface LeaderboardRow {
  rank: number;
  player_id: string;
  display_name: string | null;
  total_receipts: number;
  pass_receipts: number;
  fail_receipts: number;
  pass_rate_pct: number | null;
  last_pass_at: string | null;
  best_ocel_events: number | null;
}

export default defineEventHandler(async (event) => {
  const query = getQuery(event);
  const limit = Math.min(Number(query.limit ?? 20), 100);
  const offset = Math.max(Number(query.offset ?? 0), 0);

  const config = useRuntimeConfig(event);
  const supabaseUrl = (config.public.supabaseUrl as string) || 'http://localhost:54321';
  const serviceKey = (config.supabaseServiceRoleKey as string) || (config.public.supabaseAnonKey as string);

  if (!supabaseUrl || !serviceKey) {
    throw createError({ statusCode: 503, statusMessage: 'Supabase not configured' });
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const sb = createClient<any>(supabaseUrl, serviceKey);

  // Fetch ranked leaderboard — the trigger already maintains rank ordering
  const { data, error, count } = await sb
    .from('leaderboard')
    .select('rank, player_id, display_name, total_receipts, pass_receipts, fail_receipts, pass_rate_pct, last_pass_at, best_ocel_events', { count: 'exact' })
    .order('rank', { ascending: true })
    .range(offset, offset + limit - 1);

  if (error) throw createError({ statusCode: 500, statusMessage: error.message });

  return {
    rows: (data ?? []) as LeaderboardRow[],
    total: count ?? 0,
    offset,
    limit,
    cached_at: new Date().toISOString(),
  };
});
