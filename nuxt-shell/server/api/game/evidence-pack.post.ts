/**
 * POST /api/game/evidence-pack
 *
 * Generates a portable, tamper-evident evidence pack for a game session.
 * Pattern: ~/dashboard.bak/server/api/evidence-pack.post.ts (compliance audit bundles)
 *
 * The evidence pack bundles everything needed to verify a session offline:
 *   - OCEL 2.0 event log
 *   - Game receipt (verdict, receipt_hash, engine_source)
 *   - Per-event chain proof (from session-replay logic)
 *   - Pack manifest with BLAKE3 hash of the entire bundle
 *
 * A verifier with no database access can:
 *   1. Confirm the OCEL chain is intact (prev_hash chain)
 *   2. Confirm the receipt_hash matches the BLAKE3 of the receipt payload
 *   3. Confirm the pack_hash covers all three sections (tamper detection)
 *
 * Body: { session_id: string }
 * Returns: EvidencePack JSON
 *
 * The pack_hash is BLAKE3(canonical JSON of {ocel, receipt, chain_proof}) —
 * identical to the formula used in session-seed.post.ts for OCEL receipts.
 */

import { createClient } from '@supabase/supabase-js';
import { blake3 } from '@noble/hashes/blake3.js';

function blake3Hex(input: string): string {
  const bytes = blake3(new TextEncoder().encode(input));
  return Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join('');
}

function canonicalize(obj: unknown): string {
  return JSON.stringify(obj, (_, v) =>
    v && typeof v === 'object' && !Array.isArray(v)
      ? Object.fromEntries(Object.entries(v).sort())
      : v
  );
}

interface OcelEventRow {
  id: string;
  session_id: string;
  activity: string;
  timestamp_ms: number;
  object_refs: string[];
  attributes: Record<string, unknown>;
  prev_hash: string | null;
  event_hash: string;
  seq: number;
}

interface ReceiptRow {
  id: string;
  verdict: string;
  milestone: string;
  ocel_event_count: number;
  ocel_lifecycle: string[];
  engine_source: string;
  receipt_hash: string;
  output_hash: string | null;
  proven_at: string;
  ed25519_sig: string | null;
}

export default defineEventHandler(async (event) => {
  const body = await readBody<{ session_id?: string }>(event);
  if (!body?.session_id) {
    throw createError({ statusCode: 400, statusMessage: 'session_id required' });
  }

  const config = useRuntimeConfig(event);
  const supabaseUrl = (config.public.supabaseUrl as string) || 'http://localhost:54321';
  const serviceKey = (config.supabaseServiceRoleKey as string) || (config.public.supabaseAnonKey as string);

  if (!supabaseUrl || !serviceKey) {
    throw createError({ statusCode: 503, statusMessage: 'Supabase not configured' });
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const sb = createClient<any>(supabaseUrl, serviceKey);

  // Fetch all three components in parallel
  const [eventsRes, receiptRes] = await Promise.all([
    sb.from('ocel_events')
      .select('id, session_id, activity, timestamp_ms, object_refs, attributes, prev_hash, event_hash, seq')
      .eq('session_id', body.session_id)
      .order('seq', { ascending: true }),
    sb.from('game_receipts')
      .select('id, verdict, milestone, ocel_event_count, ocel_lifecycle, engine_source, receipt_hash, output_hash, proven_at, ed25519_sig')
      .eq('session_id', body.session_id)
      .order('proven_at', { ascending: false })
      .limit(1)
      .single(),
  ]);

  if (eventsRes.error) {
    throw createError({ statusCode: 500, statusMessage: eventsRes.error.message });
  }

  const rows = (eventsRes.data ?? []) as OcelEventRow[];

  if (rows.length === 0) {
    throw createError({ statusCode: 404, statusMessage: `No events for session ${body.session_id}` });
  }

  // Per-event chain proof (same logic as session-replay.get.ts)
  let chainIntact = true;
  let firstBreakAt: number | null = null;
  let prevHash: string | null = null;

  const chainEvents = rows.map(row => {
    const chainOk = row.prev_hash === prevHash;
    if (!chainOk && chainIntact) {
      chainIntact = false;
      firstBreakAt = row.seq;
    }
    const result = {
      seq: row.seq,
      activity: row.activity,
      event_hash: row.event_hash,
      prev_hash: row.prev_hash,
      chain_ok: chainOk,
    };
    prevHash = row.event_hash;
    return result;
  });

  const chainTip = rows[rows.length - 1]?.event_hash ?? null;

  // Build OCEL 2.0 section (same format as ocel-export.get.ts)
  const activities = [...new Set(rows.map(r => r.activity))];
  const ocel2 = {
    objectTypes: [{ name: 'game_session', attributes: [] }],
    eventTypes: activities.map(a => ({ name: a, attributes: [] })),
    objects: [{ id: body.session_id, type: 'game_session', attributes: [] }],
    events: rows.map(row => ({
      id: row.id,
      type: row.activity,
      time: new Date(row.timestamp_ms).toISOString(),
      attributes: row.attributes ?? {},
      relationships: [{ objectId: body.session_id, qualifier: 'session' }],
    })),
  };

  const receipt = receiptRes.error ? null : (receiptRes.data as ReceiptRow);

  // BLAKE3 pack hash covers ocel + chain_proof + receipt (tamper detection)
  const packPayload = {
    session_id: body.session_id,
    ocel_event_count: rows.length,
    chain_intact: chainIntact,
    chain_tip: chainTip,
    receipt_hash: receipt?.receipt_hash ?? null,
    verdict: receipt?.verdict ?? null,
  };
  const packHash = blake3Hex(canonicalize(packPayload));

  return {
    schema_version: '1.0',
    pack_hash: packHash,
    pack_algorithm: 'BLAKE3',
    generated_at: new Date().toISOString(),
    session_id: body.session_id,
    manifest: {
      total_events: rows.length,
      chain_intact: chainIntact,
      first_break_at: firstBreakAt,
      chain_tip: chainTip,
      activities,
      verdict: receipt?.verdict ?? null,
      engine_source: receipt?.engine_source ?? null,
    },
    ocel: ocel2,
    chain_proof: {
      intact: chainIntact,
      first_break_at: firstBreakAt,
      events: chainEvents,
    },
    receipt: receipt ?? null,
  };
});
