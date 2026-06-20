/**
 * POST /api/game/ocel-ingest
 *
 * Server-side OCEL event ingest — primary write path for ocel_events.
 * Accepts a pre-hashed batch from useGameSessionPersistence and:
 *   1. Inserts into ocel_events (Supabase — source of truth)
 *   2. Emits OTel spans to the local collector (non-fatal if down)
 *
 * Body: { session_id: string, events: OcelEventBatch[] }
 *   where each event already has event_hash + prev_hash computed client-side.
 *
 * Returns: { ingested: number, trace_id: string | null, session_id: string }
 *
 * The client-side BLAKE3 hashes are stored verbatim — the server does not
 * recompute them. The chain is later verified by verify_event_chain() RPC
 * which recomputes all hashes server-side and checks prev_hash threading.
 */

import { createClient } from '@supabase/supabase-js';
import { emitOtelSpans, type SpanDescriptor } from '../../utils/otlp-emitter';
import { admitBatch } from '../../utils/ocelAdmission';
import { invalidateSession } from '../../utils/conformanceCache';

interface OcelEventBatch {
  activity: string;
  timestamp_ms: number;
  object_refs: string[];
  attributes: Record<string, unknown>;
  event_hash: string;
  prev_hash: string | null;
  seq: number;
  session_id: string;
}

export default defineEventHandler(async (event) => {
  const body = await readBody<{ session_id: string; events: OcelEventBatch[] }>(event);

  if (!body?.session_id || !Array.isArray(body.events) || body.events.length === 0) {
    throw createError({ statusCode: 400, statusMessage: 'session_id and non-empty events[] required' });
  }

  const supabaseUrl = process.env.SUPABASE_URL ?? 'http://localhost:54321';
  const supabaseKey = process.env.SUPABASE_SERVICE_ROLE_KEY ?? process.env.SUPABASE_ANON_KEY ?? '';
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const supabase = createClient<any>(supabaseUrl, supabaseKey);

  // ── Cross-batch chain-tip verification ─────────────────────────────────────
  // Fetch the highest-seq event already stored for this session so we can verify
  // this batch threads correctly onto the existing chain BEFORE any insert.
  // Without this, a client sending seq [5,6,7] when [0,1,2] exists passes the
  // intra-batch contiguity check (5→6→7 valid) but leaves a gap at 3-4.
  const firstEvt = body.events[0]!;
  const { data: existingTip } = await supabase
    .from('ocel_events')
    .select('seq, event_hash')
    .eq('session_id', body.session_id)
    .order('seq', { ascending: false })
    .limit(1)
    .maybeSingle();

  if (existingTip) {
    const expectedNextSeq = (existingTip.seq as number) + 1;
    const incomingSeq = firstEvt.seq;
    if (incomingSeq > expectedNextSeq) {
      throw createError({
        statusCode: 422,
        statusMessage: `SEQ GAP: expected next seq ${expectedNextSeq}, got ${incomingSeq}. Cross-batch chain broken for session ${body.session_id}`,
      });
    }
    if (incomingSeq === expectedNextSeq && firstEvt.prev_hash !== (existingTip.event_hash as string)) {
      throw createError({
        statusCode: 422,
        statusMessage: `CHAIN THREAD BROKEN: batch[0].prev_hash does not match stored tip event_hash at seq ${existingTip.seq}`,
      });
    }
    // incomingSeq < expectedNextSeq → replay/duplicate batch; upsert ignoreDuplicates handles it
  }

  // ── Pre-Admission gate (PATQ law) ──────────────────────────────────────────
  const admission = admitBatch({
    session_id: body.session_id,
    events: body.events,
    expectedStartSeq: firstEvt.seq,
    incomingPrevHash: firstEvt.prev_hash ?? null,
  });

  if (admission.verdict === 'reject') {
    throw createError({ statusCode: admission.statusCode ?? 400, statusMessage: admission.reason ?? 'Batch rejected by admission gate' });
  }
  if (admission.verdict === 'quarantine') {
    throw createError({ statusCode: 422, statusMessage: `CHAIN TAMPER DETECTED: ${admission.reason}` });
  }

  // 1. Insert events into ocel_events (source of truth)
  const rows = body.events.map(evt => ({
    session_id: body.session_id,
    activity: evt.activity,
    timestamp_ms: evt.timestamp_ms,
    object_refs: evt.object_refs,
    attributes: evt.attributes,
    event_hash: evt.event_hash,
    prev_hash: evt.prev_hash ?? null,
    seq: evt.seq,
  }));

  // Upsert on (session_id, seq) unique constraint — idempotent-receiver pattern.
  // Duplicate event batches (retry on network error) produce the same rows, not duplicates.
  const { error: insertErr } = await supabase
    .from('ocel_events')
    .upsert(rows, { onConflict: 'session_id,seq', ignoreDuplicates: true });
  if (insertErr) {
    throw createError({ statusCode: 500, message: `ocel_events insert failed: ${insertErr.message}` });
  }

  // Invalidate conformance cache for this session — new events change fitness/generalization
  invalidateSession(body.session_id);

  // 2. Emit OTel spans (non-fatal — Supabase is source of truth)
  const descriptors: SpanDescriptor[] = body.events.map(evt => ({
    session_id: body.session_id,
    activity: evt.activity,
    timestamp_ms: evt.timestamp_ms,
    attributes: {
      'ocel.seq': evt.seq,
      'ocel.event_hash': evt.event_hash,
      'ocel.object_refs': evt.object_refs.join(','),
    },
  }));

  const { traceId } = await emitOtelSpans(descriptors).catch(() => ({ traceId: null }));

  return {
    ingested: body.events.length,
    trace_id: traceId,
    session_id: body.session_id,
  };
});
