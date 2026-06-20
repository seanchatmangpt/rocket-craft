/**
 * GET /api/game/session-state?session_id=<uuid>
 *
 * Returns the current lifecycle state of a game session.
 * Enables automated end-to-end testing to poll session state without
 * requiring direct DB access or joining multiple tables.
 *
 * State machine (mirrors sessionStateMachine.ts):
 *   Created  → is_alive=false, no session_ended_at, no receipt_hash
 *   Active   → is_alive=true
 *   Closed   → is_alive=false, session_ended_at set, no receipt_hash
 *   Proven   → is_alive=false, session_ended_at set, receipt_hash set
 *
 * Returns: {
 *   session_id: string
 *   state: 'Created' | 'Active' | 'Closed' | 'Proven' | 'NOT_FOUND'
 *   is_alive: boolean | null
 *   session_ended_at: string | null
 *   receipt_hash: string | null          — set when Proven
 *   ocel_event_count: number             — live count from ocel_events
 *   has_receipt: boolean                 — any row in game_receipts for this session
 *   last_receipt_verdict: string | null  — latest receipt verdict
 *   proven_at: string | null             — from latest PASS receipt
 * }
 */

import { createClient } from '@supabase/supabase-js';
import { deriveSessionState } from '../../utils/sessionStateMachine';

export default defineEventHandler(async (event) => {
  const query = getQuery(event);
  const sessionId = query.session_id as string | undefined;

  if (!sessionId) {
    throw createError({ statusCode: 400, statusMessage: 'session_id query param required' });
  }

  const supabaseUrl = process.env.SUPABASE_URL ?? 'http://localhost:54321';
  const supabaseKey = process.env.SUPABASE_SERVICE_ROLE_KEY ?? process.env.SUPABASE_ANON_KEY ?? '';

  if (!supabaseKey) {
    throw createError({ statusCode: 503, statusMessage: 'Supabase not configured' });
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const sb = createClient<any>(supabaseUrl, supabaseKey);

  // Fetch session row + receipt summary + OCEL event count in parallel
  const [sessionRes, receiptRes, eventCountRes] = await Promise.all([
    sb.from('game_sessions')
      .select('id, is_alive, session_ended_at, receipt_hash, created_at')
      .eq('id', sessionId)
      .single(),
    sb.from('game_receipts')
      .select('verdict, proven_at')
      .eq('session_id', sessionId)
      .order('proven_at', { ascending: false })
      .limit(1)
      .single(),
    sb.from('ocel_events')
      .select('id', { count: 'exact', head: true })
      .eq('session_id', sessionId),
  ]);

  if (sessionRes.error || !sessionRes.data) {
    return {
      session_id: sessionId,
      state: 'NOT_FOUND',
      is_alive: null,
      session_ended_at: null,
      receipt_hash: null,
      ocel_event_count: 0,
      has_receipt: false,
      last_receipt_verdict: null,
      proven_at: null,
    };
  }

  const session = sessionRes.data as {
    id: string;
    is_alive: boolean;
    session_ended_at: string | null;
    receipt_hash: string | null;
    created_at: string;
  };

  const state = deriveSessionState({
    id: session.id,
    is_alive: session.is_alive,
    session_ended_at: session.session_ended_at,
    receipt_hash: session.receipt_hash,
  });

  const receipt = receiptRes.data as { verdict: string; proven_at: string } | null;
  const ocelCount = eventCountRes.count ?? 0;

  return {
    session_id: sessionId,
    state,
    is_alive: session.is_alive,
    session_ended_at: session.session_ended_at,
    receipt_hash: session.receipt_hash,
    ocel_event_count: ocelCount,
    has_receipt: receipt !== null,
    last_receipt_verdict: receipt?.verdict ?? null,
    proven_at: receipt?.proven_at ?? null,
  };
});
