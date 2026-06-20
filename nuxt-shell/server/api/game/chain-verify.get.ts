/**
 * GET /api/game/chain-verify?session_id=<uuid>
 *
 * Calls the `verify_event_chain` Postgres RPC to validate that the ocel_events
 * hash chain for the given session is intact, and adds:
 *   - merkle_root: BLAKE3 Merkle tree root over all event_hash values
 *     (ordering proof via prev_hash chain + membership proof via Merkle root)
 *   - event_count: number of events covered by the proof
 *
 * Van der Aalst doctrine: server-side proof, not browser-side assertion.
 * Returns { overall, sessions_checked, breaks, rows, merkle_root, event_count }
 */
import { createClient } from '@supabase/supabase-js';
import { computeMerkleRoot } from '../../utils/merkle';
import { checkConformance } from '../../utils/processMining';

const DECLARED_LIFECYCLE = [
  'GameSessionStarted',
  'InputAdmitted',
  'FrameRendered',
  'GameSessionClosed',
];

export default defineEventHandler(async (event) => {
  const query = getQuery(event);
  const sessionId = typeof query.session_id === 'string' ? query.session_id : null;

  const supabaseUrl = (process.env.SUPABASE_URL) || 'http://localhost:54321';
  const serviceKey = (process.env.SUPABASE_SERVICE_ROLE_KEY) || (process.env.SUPABASE_ANON_KEY) || '';

  if (!supabaseUrl || !serviceKey) {
    throw createError({
      statusCode: 503,
      message: 'Supabase not configured — set SUPABASE_URL and SUPABASE_SERVICE_ROLE_KEY',
    });
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const supabase = createClient<any>(supabaseUrl, serviceKey);

  const [rpcResult, hashesResult, eventsResult] = await Promise.all([
    sessionId
      ? supabase.rpc('verify_event_chain', { p_session_id: sessionId })
      : supabase.rpc('verify_event_chain', {}),
    sessionId
      ? supabase
          .from('ocel_events')
          .select('event_hash, seq')
          .eq('session_id', sessionId)
          .order('seq', { ascending: true })
      : Promise.resolve({ data: [] as Array<{ event_hash: string; seq: number }>, error: null }),
    // Fetch full event rows for conformance analysis
    sessionId
      ? supabase
          .from('ocel_events')
          .select('activity, timestamp_ms, seq')
          .eq('session_id', sessionId)
          .order('seq', { ascending: true })
      : Promise.resolve({ data: [] as Array<{ activity: string; timestamp_ms: number; seq: number }>, error: null }),
  ]);

  if (rpcResult.error) {
    throw createError({ statusCode: 500, message: rpcResult.error.message });
  }

  const rows = (rpcResult.data ?? []) as Array<{
    ok: boolean;
    message: string;
    broken_at: number | null;
    session_id: string;
  }>;

  const allOk = rows.every((r) => r.ok);
  const breaks = rows.filter((r) => !r.ok);

  // Merkle root over event_hash values (ordered by seq)
  const eventHashes = ((hashesResult.data ?? []) as Array<{ event_hash: string }>)
    .map(r => r.event_hash)
    .filter(Boolean);
  const merkleRoot = computeMerkleRoot(eventHashes);

  // Process conformance: fitness, precision, simplicity against declared lifecycle
  const miningEvents = (eventsResult.data ?? []) as Array<{ activity: string; timestamp_ms: number; seq: number }>;
  const conformance = checkConformance(miningEvents, DECLARED_LIFECYCLE);

  return {
    overall: allOk ? 'PASS' : 'FAIL',
    sessions_checked: rows.length,
    breaks,
    rows,
    merkle_root: merkleRoot,
    event_count: eventHashes.length,
    // Van der Aalst 4-dimension conformance: fitness, precision, simplicity, generalization
    conformance: {
      fitness: conformance.fitness,
      precision: conformance.precision,
      simplicity: conformance.simplicity,
      generalization: conformance.generalization,
      overall_score: conformance.overall_score,
      variants_discovered: conformance.variants_discovered,
      deviation_points: conformance.deviation_points,
    },
  };
});
