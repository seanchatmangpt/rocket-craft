/**
 * GET /api/game/receipts
 *
 * Paginated game_receipts list — replaces the client-side .from('game_receipts')
 * query in receipts.vue. Service role key never leaves the server.
 *
 * Query params:
 *   limit         — max rows (default 50, max 200)
 *   offset        — pagination offset (default 0)
 *   verdict       — filter by verdict: PASS | FAIL | PENDING (optional)
 *   engine_source — filter by engine source e.g. rocket_cli (optional)
 *   session_id    — filter to a specific session (optional)
 *
 * Returns:
 *   { rows: ReceiptRow[], total: number, offset, limit }
 */

import { createClient } from '@supabase/supabase-js';

export default defineEventHandler(async (event) => {
  const query = getQuery(event);
  const limit = Math.min(Number(query.limit ?? 50), 200);
  const offset = Math.max(Number(query.offset ?? 0), 0);
  const verdict = typeof query.verdict === 'string' ? query.verdict : null;
  const engineSource = typeof query.engine_source === 'string' ? query.engine_source : null;
  const sessionId = typeof query.session_id === 'string' ? query.session_id : null;

  const config = useRuntimeConfig(event);
  const supabaseUrl = (config.public.supabaseUrl as string) || 'http://localhost:54321';
  const serviceKey = (config.supabaseServiceRoleKey as string) || (config.public.supabaseAnonKey as string);

  if (!supabaseUrl || !serviceKey) {
    throw createError({ statusCode: 503, statusMessage: 'Supabase not configured' });
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const sb = createClient<any>(supabaseUrl, serviceKey);

  let q = sb
    .from('game_receipts')
    .select(
      'id, session_id, verdict, milestone, ocel_event_count, ocel_lifecycle, engine_source, receipt_hash, output_hash, proven_at, ed25519_sig, payload',
      { count: 'exact' }
    )
    .order('proven_at', { ascending: false })
    .range(offset, offset + limit - 1);

  if (verdict) q = q.eq('verdict', verdict);
  if (engineSource) q = q.eq('engine_source', engineSource);
  if (sessionId) q = q.eq('session_id', sessionId);

  const { data, error, count } = await q;
  if (error) throw createError({ statusCode: 500, statusMessage: error.message });

  return {
    rows: data ?? [],
    total: count ?? 0,
    offset,
    limit,
  };
});
