/**
 * POST /api/game/qa-cycle
 *
 * Autonomous QA cycle for a game session — inspired by AutonomicQAEngine.
 * Checks four invariants, computes a BLAKE3 cycle receipt hash, and returns
 * an overall health verdict.
 *
 * Body: { session_id: string }
 *
 * Returns:
 *   {
 *     session_id, cycle_id, overall, checks_passed, checks_total,
 *     results, cycle_receipt_hash, started_at, completed_at
 *   }
 */

import { createClient } from '@supabase/supabase-js';
import { blake3 } from '@noble/hashes/blake3.js';
import { computeMerkleRoot } from '../../utils/merkle';
import { classifyOverall, buildCycleResult } from '../../utils/qaCycle';

function blake3Hex(input: string): string {
  const bytes = blake3(new TextEncoder().encode(input));
  return Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join('');
}

function canonicalize(obj: unknown): string {
  return JSON.stringify(obj, (_, v) =>
    v && typeof v === 'object' && !Array.isArray(v)
      ? Object.fromEntries(Object.entries(v as Record<string, unknown>).sort())
      : v
  );
}

function uuid(): string {
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, c => {
    const r = Math.random() * 16 | 0;
    const v = c === 'x' ? r : (r & 0x3 | 0x8);
    return v.toString(16);
  });
}

export default defineEventHandler(async (event) => {
  const body = await readBody<{ session_id?: string }>(event);
  if (!body?.session_id) {
    throw createError({ statusCode: 400, statusMessage: 'session_id required' });
  }

  const supabaseUrl = process.env.SUPABASE_URL ?? 'http://localhost:54321';
  const serviceKey = process.env.SUPABASE_SERVICE_ROLE_KEY ?? process.env.SUPABASE_ANON_KEY ?? '';

  if (!serviceKey) {
    throw createError({ statusCode: 503, statusMessage: 'Supabase not configured' });
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const sb = createClient<any>(supabaseUrl, serviceKey);

  const startedAt = new Date().toISOString();
  const cycleId = uuid();

  // Run all checks in parallel
  const [chainRes, eventsRes, receiptRes] = await Promise.all([
    sb.rpc('verify_event_chain', { p_session_id: body.session_id }),
    sb.from('ocel_events')
      .select('activity, event_hash, seq, timestamp_ms')
      .eq('session_id', body.session_id)
      .order('seq', { ascending: true }),
    sb.from('game_receipts')
      .select('engine_source')
      .eq('session_id', body.session_id)
      .order('proven_at', { ascending: false })
      .limit(1)
      .single(),
  ]);

  const chainOk: boolean | null = chainRes.data?.ok ?? null;
  const eventRows = (eventsRes.data ?? []) as Array<{
    activity: string;
    event_hash: string;
    seq: number;
    timestamp_ms: number;
  }>;
  const activities = [...new Set(eventRows.map(r => r.activity))];
  const eventHashes = eventRows.map(r => r.event_hash).filter(Boolean);
  const merkleRoot = computeMerkleRoot(eventHashes);
  const engineSource: string | null = receiptRes.data?.engine_source ?? null;

  // Build MiningEvent array for Van der Aalst conformance scoring
  const miningEvents = eventRows.map(r => ({
    activity: r.activity,
    timestamp_ms: r.timestamp_ms ?? 0,
    seq: r.seq ?? 0,
  }));

  const results = buildCycleResult({ chainOk, activities, engineSource, eventHashes, merkleRoot, miningEvents });

  const overall = classifyOverall(results);
  const checksPassed = results.filter(r => r.passed).length;
  const checksTotal = results.length;

  const completedAt = new Date().toISOString();

  // BLAKE3 cycle receipt hash
  const cycleReceiptHash = blake3Hex(canonicalize({
    session_id: body.session_id,
    cycle_started_at: startedAt,
    results,
  }));

  return {
    session_id: body.session_id,
    cycle_id: cycleId,
    overall,
    checks_passed: checksPassed,
    checks_total: checksTotal,
    results,
    cycle_receipt_hash: cycleReceiptHash,
    started_at: startedAt,
    completed_at: completedAt,
  };
});
