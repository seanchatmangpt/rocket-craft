/**
 * POST /api/game/session
 *
 * Creates a new game_sessions row + initial PENDING game_receipts row.
 * The receipt is needed so receipt-finalize can UPDATE it to PROVEN later.
 * Without an initial receipt row, the finalize UPDATE affects 0 rows silently.
 *
 * Body: { browser_session_id: string, engine_source?: string }
 * Returns: { session_id: string, receipt_hash: string, started_at: string }
 */

import { createClient } from '@supabase/supabase-js';
import { blake3 } from '@noble/hashes/blake3.js';

function blake3Hex(input: string): string {
  const bytes = blake3(new TextEncoder().encode(input));
  return Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join('');
}

export default defineEventHandler(async (event) => {
  const body = await readBody(event).catch(() => ({}));
  const browserSessionId: string | undefined = body?.browser_session_id;
  const engineSource: string = body?.engine_source ?? 'browser';

  if (!browserSessionId) {
    throw createError({ statusCode: 400, message: 'browser_session_id is required' });
  }

  const supabaseUrl = process.env.SUPABASE_URL ?? 'http://localhost:54321';
  const supabaseKey = process.env.SUPABASE_SERVICE_ROLE_KEY ?? process.env.SUPABASE_ANON_KEY ?? '';
  const supabase = createClient(supabaseUrl, supabaseKey);

  const startedAt = new Date().toISOString();

  // Insert game_sessions row
  const { data, error } = await supabase
    .from('game_sessions')
    .insert({
      player_id: null,
      session_started_at: startedAt,
      session_ended_at: null,
      engine_source: engineSource,
      is_alive: true,
      ocel_event_count: 0,
      receipt_hash: null,
      metadata: { browser_session_id: browserSessionId },
    })
    .select('id')
    .single();

  if (error || !data) {
    throw createError({ statusCode: 500, message: error?.message ?? 'session insert failed' });
  }

  const sessionId = data.id as string;

  // Create initial PENDING receipt so receipt-finalize can UPDATE it to PROVEN.
  // Without this row, the finalize UPDATE silently affects 0 rows.
  const receiptHash = blake3Hex(JSON.stringify({
    session_id: sessionId,
    started_at: startedAt,
    milestone: 'BrowserSessionCreated',
  }));

  await supabase.from('game_receipts').insert({
    session_id: sessionId,
    receipt_hash: receiptHash,
    verdict: 'PENDING',
    engine_source: engineSource,
    ocel_event_count: 0,
    ocel_lifecycle: [],
    milestone: 'BrowserSessionCreated',
    proven_at: null,
  }).select('id').single();
  // Non-fatal if receipt insert fails — session is still valid; receipt-finalize
  // will fail gracefully with a 404 on chain-verify rather than silently no-op.

  return { session_id: sessionId, receipt_hash: receiptHash, started_at: startedAt };
});
