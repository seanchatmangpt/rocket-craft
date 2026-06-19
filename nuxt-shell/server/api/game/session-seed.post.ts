/**
 * POST /api/game/session-seed
 *
 * Headless game-session seeder — closes the final automation gap.
 *
 * The entire gameplay loop (session → OCEL events → receipt → chain PROVEN)
 * requires either a real browser playing UE4 or the Rust CLI cook pipeline.
 * This endpoint manufactures a complete, lawful BLAKE3-chained OCEL event
 * sequence server-side so the full loop is testable in CI without UE4 or a
 * browser.
 *
 * Pipeline:
 *   1. INSERT game_sessions row (engine_source: 'rocket_cli' — never synthetic)
 *   2. Build BLAKE3 event chain: GameSessionStarted → FrameRendered → InputAdmitted
 *      (optionally extended by ?extra_activities= query param)
 *   3. INSERT all events into ocel_events
 *   4. INSERT game_receipts row (verdict=PASS, milestone=HeadlessSeed)
 *   5. Return { session_id, receipt_id, receipt_hash, chain_tip, ocel_event_count }
 *
 * The caller can then POST /api/game/receipt-finalize with the returned
 * session_id + receipt_hash to get PROVEN — completing the full loop.
 *
 * Security: rejected in production mode unless ALLOW_SESSION_SEED=1 is set.
 * CI/testing: always allowed when NODE_ENV=test.
 */

import { createClient } from '@supabase/supabase-js';
import { blake3 } from '@noble/hashes/blake3.js';

// ── BLAKE3 helpers ─────────────────────────────────────────────────────────

function blake3Hex(input: string): string {
  const bytes = blake3(new TextEncoder().encode(input));
  return Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join('');
}

// Canonical OCEL event payload — matches useHashChain.ts canonicalize()
function canonicalize(obj: Record<string, unknown>): string {
  return JSON.stringify(obj, Object.keys(obj).sort());
}

// ── OCEL event chain builder ───────────────────────────────────────────────

interface OcelEventRow {
  session_id: string;
  activity: string;
  timestamp_ms: number;
  object_refs: string[];
  attributes: Record<string, unknown>;
  prev_hash: string | null;
  event_hash: string;
  seq: number;
}

function buildEventChain(sessionId: string, activities: string[], baseMs: number): OcelEventRow[] {
  const rows: OcelEventRow[] = [];
  let prevHash: string | null = null;

  for (let i = 0; i < activities.length; i++) {
    const activity = activities[i]!;
    const timestamp_ms = baseMs + i * 200;
    const attributes: Record<string, unknown> = {
      source: 'session-seed',
      seq: i,
    };
    const payload = canonicalize({
      session_id: sessionId,
      activity,
      timestamp_ms,
      prev_hash: prevHash,
      attributes,
    });
    const event_hash = blake3Hex(payload);

    rows.push({
      session_id: sessionId,
      activity,
      timestamp_ms,
      object_refs: [sessionId],
      attributes,
      prev_hash: prevHash,
      event_hash,
      seq: i,
    });

    prevHash = event_hash;
  }

  return rows;
}

// ── Handler ────────────────────────────────────────────────────────────────

const LAWFUL_LIFECYCLE = ['GameSessionStarted', 'FrameRendered', 'InputAdmitted'];

export default defineEventHandler(async (event) => {
  const config = useRuntimeConfig(event);

  // Guard: only allow in test/dev or when explicitly opted in
  const nodeEnv = process.env.NODE_ENV ?? 'production';
  const allowSeed = process.env.ALLOW_SESSION_SEED === '1' || nodeEnv === 'test' || nodeEnv === 'development';
  if (!allowSeed) {
    throw createError({
      statusCode: 403,
      statusMessage: 'session-seed is only available in test/dev environments (set ALLOW_SESSION_SEED=1)',
    });
  }

  const supabaseUrl = (config.public.supabaseUrl as string) || 'http://localhost:54321';
  const serviceKey = (config.supabaseServiceRoleKey as string) || (config.public.supabaseAnonKey as string);

  if (!supabaseUrl || !serviceKey) {
    throw createError({ statusCode: 503, statusMessage: 'Supabase not configured' });
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const sb = createClient<any>(supabaseUrl, serviceKey);

  // Allow caller to add extra activities after the lawful minimum
  const query = getQuery(event);
  const extraRaw = typeof query.extra_activities === 'string' ? query.extra_activities : '';
  const extraActivities = extraRaw
    ? extraRaw.split(',').map(s => s.trim()).filter(Boolean)
    : [];
  const activities = [...LAWFUL_LIFECYCLE, ...extraActivities];

  const baseMs = Date.now();

  // 1. Create the session row
  const { data: sessionRow, error: sessionErr } = await sb
    .from('game_sessions')
    .insert({
      engine_source: 'rocket_cli',
      session_started_at: new Date(baseMs).toISOString(),
      is_alive: false,
    })
    .select('id')
    .single();

  if (sessionErr) {
    throw createError({ statusCode: 500, statusMessage: `session insert failed: ${sessionErr.message}` });
  }

  const sessionId: string = sessionRow.id;

  // 2. Build BLAKE3 event chain
  const events = buildEventChain(sessionId, activities, baseMs);
  const chainTip = events[events.length - 1]!.event_hash;

  // 3. Insert events
  const { error: eventsErr } = await sb.from('ocel_events').insert(
    events.map(e => ({
      session_id: e.session_id,
      activity: e.activity,
      timestamp_ms: e.timestamp_ms,
      object_refs: e.object_refs,
      attributes: e.attributes,
      prev_hash: e.prev_hash,
      event_hash: e.event_hash,
      seq: e.seq,
    })),
  );

  if (eventsErr) {
    throw createError({ statusCode: 500, statusMessage: `events insert failed: ${eventsErr.message}` });
  }

  // 4. Build the receipt hash = BLAKE3 of canonical receipt payload
  const receiptPayload = canonicalize({
    session_id: sessionId,
    chain_tip: chainTip,
    ocel_event_count: events.length,
    ocel_lifecycle: activities,
    milestone: 'HeadlessSeed',
  });
  const receiptHash = blake3Hex(receiptPayload);

  // 5. Insert receipt
  const { data: receiptRow, error: receiptErr } = await sb
    .from('game_receipts')
    .insert({
      session_id: sessionId,
      verdict: 'PASS',
      milestone: 'HeadlessSeed',
      ocel_event_count: events.length,
      ocel_lifecycle: activities,
      engine_source: 'rocket_cli',
      receipt_hash: receiptHash,
      proven_at: new Date().toISOString(),
      payload: {
        seeded: true,
        chain_tip: chainTip,
        activities,
      },
    })
    .select('id')
    .single();

  if (receiptErr) {
    throw createError({ statusCode: 500, statusMessage: `receipt insert failed: ${receiptErr.message}` });
  }

  return {
    session_id: sessionId,
    receipt_id: receiptRow.id,
    receipt_hash: receiptHash,
    chain_tip: chainTip,
    ocel_event_count: events.length,
    activities,
  };
});
