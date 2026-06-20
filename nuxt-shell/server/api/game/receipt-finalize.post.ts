/**
 * POST /api/game/receipt-finalize
 *
 * Gap 2 fix: Bundles chain verification + receipt hash match into a single
 * automated proof endpoint, callable from Playwright after a game session.
 *
 * Replaces the manual "go to /pipeline and look at the chain-verify section"
 * step with a machine-checkable assertion:
 *   chain intact ∧ receipt_hash == chain_tip → PROVEN
 *
 * Body: { session_id: string; receipt_hash: string }
 * Returns: {
 *   session_id,
 *   chain_verified: boolean,       // verify_event_chain RPC returned ok=true
 *   chain_tip_matches_hash: boolean, // chain tip BLAKE3 == receipt_hash
 *   broken_at: number | null,      // event seq where chain broke (if any)
 *   verdict: 'PROVEN' | 'CHAIN_BROKEN' | 'HASH_MISMATCH' | 'NO_EVENTS',
 * }
 */

import { createClient } from '@supabase/supabase-js';

interface FinalizeBody {
  session_id: string;
  receipt_hash: string;
  /** Optional: update the matching game_receipts row with the proven verdict */
  update_receipt?: boolean;
}

export default defineEventHandler(async (event) => {
  const body = await readBody<FinalizeBody>(event);
  if (!body?.session_id || !body?.receipt_hash) {
    throw createError({ statusCode: 400, statusMessage: 'session_id and receipt_hash required' });
  }

  const config = useRuntimeConfig(event);
  const supabaseUrl = config.public.supabaseUrl as string;
  const serviceKey = config.supabaseServiceRoleKey as string;

  if (!supabaseUrl || !serviceKey) {
    throw createError({ statusCode: 503, statusMessage: 'Supabase not configured' });
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const sb = createClient<any>(supabaseUrl, serviceKey);

  // Step 1: verify hash chain integrity via the RPC
  const { data: chainRows, error: chainErr } = await sb.rpc('verify_event_chain', {
    p_session_id: body.session_id,
  });

  if (chainErr) {
    throw createError({ statusCode: 500, statusMessage: `Chain verify RPC failed: ${chainErr.message}` });
  }

  const rows = (chainRows ?? []) as Array<{
    ok: boolean;
    message: string;
    broken_at: number | null;
    session_id: string;
  }>;

  if (rows.length === 0) {
    return {
      session_id: body.session_id,
      chain_verified: false,
      chain_tip_matches_hash: false,
      broken_at: null,
      verdict: 'NO_EVENTS',
    };
  }

  const sessionRow = rows.find(r => r.session_id === body.session_id) ?? rows[0]!;
  const chainVerified = sessionRow.ok;
  const brokenAt = sessionRow.broken_at ?? null;

  if (!chainVerified) {
    return {
      session_id: body.session_id,
      chain_verified: false,
      chain_tip_matches_hash: false,
      broken_at: brokenAt,
      verdict: 'CHAIN_BROKEN',
    };
  }

  // Step 2: fetch the chain tip (hash of the last event in sequence)
  const { data: tipRow, error: tipErr } = await sb
    .from('ocel_events')
    .select('event_hash, seq')
    .eq('session_id', body.session_id)
    .order('seq', { ascending: false })
    .limit(1)
    .single();

  if (tipErr || !tipRow) {
    return {
      session_id: body.session_id,
      chain_verified: true,
      chain_tip_matches_hash: false,
      broken_at: null,
      verdict: 'NO_EVENTS',
    };
  }

  const chainTipMatches = tipRow.event_hash === body.receipt_hash;
  const verdict = chainTipMatches ? 'PROVEN' : 'HASH_MISMATCH';
  const provenAt = new Date().toISOString();

  // Stamp both game_receipts AND game_sessions when PROVEN.
  // Previously only game_receipts was updated — sessions stayed is_alive=false
  // indefinitely with no proven_at, making it impossible to query "proven sessions"
  // without a full join. Now a single game_sessions row captures the terminal state.
  if (body.update_receipt || verdict === 'PROVEN') {
    // Update game_receipts verdict
    await sb
      .from('game_receipts')
      .update({ verdict, proven_at: provenAt })
      .eq('session_id', body.session_id)
      .eq('receipt_hash', body.receipt_hash);

    // Stamp game_sessions with receipt_hash + proven_at (terminal state transition)
    // This enables O(1) "is this session proven?" without joining game_receipts.
    if (verdict === 'PROVEN') {
      await sb
        .from('game_sessions')
        .update({
          receipt_hash: body.receipt_hash,
          session_ended_at: provenAt,
          last_proof_at: provenAt,
          is_alive: false,
        })
        .eq('id', body.session_id)
        .is('receipt_hash', null); // only stamp once — do not overwrite an existing proof
    }
  }

  return {
    session_id: body.session_id,
    chain_verified: true,
    chain_tip_matches_hash: chainTipMatches,
    broken_at: null,
    verdict,
    chain_tip: tipRow.event_hash,
    receipt_hash: body.receipt_hash,
    tip_seq: tipRow.seq,
    proven_at: verdict === 'PROVEN' ? provenAt : null,
  };
});
