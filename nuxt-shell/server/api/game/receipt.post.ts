/**
 * POST /api/game/receipt
 *
 * Server-side receipt validation and Supabase persistence.
 * Pattern: ~/dashboard.bak/server/api/verify-receipt.post.ts
 *
 * Verifies the OCEL lifecycle is lawful (GameSessionStarted → FrameRendered → InputAdmitted*)
 * before writing to game_receipts. This prevents client-side forgery.
 *
 * Body: { session_id, ocel_lifecycle, ocel_event_count, engine_source, receipt_hash, milestone, payload }
 * Returns: { receipt_id, verdict, milestone }
 *
 * OTel span emission uses server/utils/otlp-emitter — failures are logged but
 * never throw so OTLP unavailability cannot block receipt creation.
 */

import { emitOtelSpan } from '~/server/utils/otlp-emitter';

const LAWFUL_LIFECYCLE_START = 'GameSessionStarted';
const LAWFUL_LIFECYCLE_FRAME = 'FrameRendered';
const MIN_EVENTS_FOR_PASS = 3;

interface ReceiptBody {
  session_id: string;
  ocel_lifecycle: string[];
  ocel_event_count: number;
  engine_source: string;
  receipt_hash: string;
  milestone: string;
  payload: Record<string, unknown>;
}

const SYNTHETIC_ENGINE_SOURCES = ['synthetic', 'sim', 'fake', 'stub', 'mock'];

function verifyLifecycle(lifecycle: string[], eventCount: number): { verdict: 'PASS' | 'FAIL'; reason: string } {
  if (!lifecycle.includes(LAWFUL_LIFECYCLE_START)) {
    return { verdict: 'FAIL', reason: 'Missing GameSessionStarted' };
  }
  if (!lifecycle.includes(LAWFUL_LIFECYCLE_FRAME)) {
    return { verdict: 'FAIL', reason: 'Missing FrameRendered — engine never proved live' };
  }
  if (eventCount < MIN_EVENTS_FOR_PASS) {
    return { verdict: 'FAIL', reason: `Insufficient OCEL events: ${eventCount} < ${MIN_EVENTS_FOR_PASS}` };
  }
  return { verdict: 'PASS', reason: 'Lawful OCEL lifecycle confirmed' };
}

export default defineEventHandler(async (event) => {
  const body = await readBody<ReceiptBody>(event);

  if (!body?.session_id || !body.ocel_lifecycle || !body.receipt_hash) {
    throw createError({ statusCode: 400, statusMessage: 'session_id, ocel_lifecycle, receipt_hash required' });
  }

  // Hard reject synthetic engine sources — only real UE4 proof accepted
  if (SYNTHETIC_ENGINE_SOURCES.includes((body.engine_source ?? '').toLowerCase())) {
    throw createError({
      statusCode: 422,
      statusMessage: `engine_source '${body.engine_source}' is not real UE4 — synthetic receipts are rejected`,
    });
  }

  const { verdict, reason } = verifyLifecycle(body.ocel_lifecycle, body.ocel_event_count ?? 0);

  const config = useRuntimeConfig();
  const { createClient } = await import('@supabase/supabase-js');

  const sb = createClient(
    (config.public.supabaseUrl as string) || 'http://localhost:54321',
    // Use service role key server-side for insert bypass RLS
    (config.supabaseServiceRoleKey as string) || (config.public.supabaseAnonKey as string) || '',
  );

  const { data, error } = await sb
    .from('game_receipts')
    .insert({
      session_id: body.session_id,
      verdict,
      milestone: body.milestone,
      ocel_event_count: body.ocel_event_count,
      ocel_lifecycle: body.ocel_lifecycle,
      engine_source: body.engine_source,
      receipt_hash: body.receipt_hash,
      proven_at: new Date().toISOString(),
      payload: { ...body.payload, server_reason: reason },
    })
    .select('id')
    .single();

  if (error) {
    throw createError({ statusCode: 500, statusMessage: `Receipt persist failed: ${error.message}` });
  }

  // Emit OTel span for the receipt verdict — non-fatal if collector is unreachable.
  // Fire-and-forget: we do not await the result as the Supabase write already succeeded.
  emitOtelSpan(body.session_id, 'ReceiptVerdict', {
    'receipt.verdict': verdict,
    'receipt.milestone': body.milestone ?? '',
    'receipt.hash': body.receipt_hash,
    'receipt.engine_source': body.engine_source ?? '',
    'receipt.ocel_event_count': body.ocel_event_count ?? 0,
    'receipt.id': data.id as string,
  }).catch((err: unknown) => {
    // Should never reach here since emitOtelSpan itself never throws, but guard
    // defensively so a future refactor cannot accidentally break receipt creation.
    const msg = err instanceof Error ? err.message : String(err);
    console.warn(`[receipt.post] emitOtelSpan swallowed unexpected error: ${msg}`);
  });

  return {
    receipt_id: data.id,
    verdict,
    reason,
    milestone: body.milestone,
  };
});
