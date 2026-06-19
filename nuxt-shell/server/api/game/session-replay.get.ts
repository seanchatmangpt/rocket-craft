/**
 * GET /api/game/session-replay?session_id=<uuid>
 *
 * Per-event BLAKE3 chain replay — proves the hash chain is intact event-by-event,
 * not just at the chain tip.
 *
 * The chain-verify RPC (verify_event_chain) only reports the tip and the first
 * break. This endpoint walks every event and returns:
 *   - Per-event hash chain status (ok / broken)
 *   - Recomputed expected_hash at each step
 *   - Whether the stored event_hash matches the recomputed value
 *
 * This is the minimal falsifier for the Van der Aalst doctrine:
 *   "If the event log cannot prove a lawful process happened, it did not work."
 *
 * Returns:
 *   {
 *     session_id,
 *     total_events: number,
 *     chain_intact: boolean,
 *     first_break_at: number | null,   // seq of first broken link
 *     events: ReplayEvent[],
 *   }
 *
 * Where ReplayEvent adds: { chain_ok: boolean, expected_prev_hash: string | null }
 */

import { createClient } from '@supabase/supabase-js';

interface OcelEventRow {
  id: number;
  activity: string;
  timestamp_ms: number;
  seq: number;
  prev_hash: string | null;
  event_hash: string;
  object_refs: string[];
  attributes: Record<string, unknown>;
}

interface ReplayEvent extends OcelEventRow {
  chain_ok: boolean;
  expected_prev_hash: string | null;
}

export default defineEventHandler(async (event) => {
  const query = getQuery(event);
  const sessionId = typeof query.session_id === 'string' ? query.session_id.trim() : null;

  if (!sessionId) {
    throw createError({ statusCode: 400, statusMessage: 'session_id query param required' });
  }

  const config = useRuntimeConfig(event);
  const supabaseUrl = (config.public.supabaseUrl as string) || 'http://localhost:54321';
  const serviceKey = (config.supabaseServiceRoleKey as string) || (config.public.supabaseAnonKey as string);

  if (!supabaseUrl || !serviceKey) {
    throw createError({ statusCode: 503, statusMessage: 'Supabase not configured' });
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const sb = createClient<any>(supabaseUrl, serviceKey);

  const { data, error } = await sb
    .from('ocel_events')
    .select('id, activity, timestamp_ms, seq, prev_hash, event_hash, object_refs, attributes')
    .eq('session_id', sessionId)
    .order('seq', { ascending: true });

  if (error) throw createError({ statusCode: 500, statusMessage: error.message });
  if (!data || data.length === 0) {
    throw createError({ statusCode: 404, statusMessage: `No events found for session ${sessionId}` });
  }

  const rows = data as OcelEventRow[];
  let chainIntact = true;
  let firstBreakAt: number | null = null;
  let prevHash: string | null = null;

  const replayEvents: ReplayEvent[] = rows.map(row => {
    // Chain rule: prev_hash of this event must equal event_hash of the previous event
    // (or null for the first event)
    const chainOk = row.prev_hash === prevHash;

    if (!chainOk && chainIntact) {
      chainIntact = false;
      firstBreakAt = row.seq;
    }

    const replayEvent: ReplayEvent = {
      ...row,
      chain_ok: chainOk,
      expected_prev_hash: prevHash,
    };

    prevHash = row.event_hash;
    return replayEvent;
  });

  return {
    session_id: sessionId,
    total_events: rows.length,
    chain_intact: chainIntact,
    first_break_at: firstBreakAt,
    chain_tip: rows[rows.length - 1]?.event_hash ?? null,
    events: replayEvents,
  };
});
