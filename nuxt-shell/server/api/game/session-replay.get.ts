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
import { blake3 } from '@noble/hashes/blake3.js';

interface OcelEventRow {
  id: number;
  activity: string;
  timestamp_ms: number;
  seq: number;
  prev_hash: string | null;
  event_hash: string;
  object_refs: string[];
  attributes: Record<string, unknown>;
  session_id?: string;
}

interface ReplayEvent extends OcelEventRow {
  chain_ok: boolean;
  expected_prev_hash: string | null;
  /** Server recomputed hash from canonical payload — null if session_id not stored on event row */
  recomputed_hash: string | null;
  /** Whether stored event_hash matches server-side recomputation */
  hash_convergent: boolean | null;
}

function blake3Hex(input: string): string {
  const bytes = blake3(new TextEncoder().encode(input));
  return Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join('');
}

function canonicalize(obj: Record<string, unknown>): string {
  return JSON.stringify(obj, Object.keys(obj).sort());
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
    .select('id, session_id, activity, timestamp_ms, seq, prev_hash, event_hash, object_refs, attributes')
    .eq('session_id', sessionId)
    .order('seq', { ascending: true });

  if (error) throw createError({ statusCode: 500, statusMessage: error.message });
  if (!data || data.length === 0) {
    throw createError({ statusCode: 404, statusMessage: `No events found for session ${sessionId}` });
  }

  const rows = data as OcelEventRow[];
  let chainIntact = true;
  let hashConvergent = true;
  let firstBreakAt: number | null = null;
  let firstConvergenceFailAt: number | null = null;
  let prevHash: string | null = null;

  const replayEvents: ReplayEvent[] = rows.map(row => {
    // Chain rule: prev_hash of this event must equal event_hash of the previous event
    const chainOk = row.prev_hash === prevHash;
    if (!chainOk && chainIntact) {
      chainIntact = false;
      firstBreakAt = row.seq;
    }

    // Convergence rule: recompute BLAKE3(canonical_payload) and compare to stored event_hash.
    // This catches tampered event_hash fields that still thread the chain correctly.
    let recomputedHash: string | null = null;
    let isHashConvergent: boolean | null = null;
    const evtSessionId = row.session_id ?? sessionId;
    try {
      const payload = canonicalize({
        session_id: evtSessionId,
        activity: row.activity,
        timestamp_ms: row.timestamp_ms,
        prev_hash: row.prev_hash,
        attributes: row.attributes ?? {},
      });
      recomputedHash = blake3Hex(payload);
      isHashConvergent = recomputedHash === row.event_hash;
    } catch {
      isHashConvergent = null; // cannot verify
    }

    if (isHashConvergent === false && hashConvergent) {
      hashConvergent = false;
      firstConvergenceFailAt = row.seq;
    }

    const replayEvent: ReplayEvent = {
      ...row,
      chain_ok: chainOk,
      expected_prev_hash: prevHash,
      recomputed_hash: recomputedHash,
      hash_convergent: isHashConvergent,
    };

    prevHash = row.event_hash;
    return replayEvent;
  });

  return {
    session_id: sessionId,
    total_events: rows.length,
    chain_intact: chainIntact,
    hash_convergent: hashConvergent,
    first_break_at: firstBreakAt,
    first_convergence_fail_at: firstConvergenceFailAt,
    chain_tip: rows[rows.length - 1]?.event_hash ?? null,
    // Tamper verdict: chain threading AND content convergence must both hold
    tamper_evident: chainIntact && hashConvergent,
    events: replayEvents,
  };
});
