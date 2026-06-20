/**
 * GET /api/game/wasm-crosscheck?output_hash=<blake3-hex>
 *
 * Cook-to-game binary cross-check API (Gap 6 server side).
 *
 * The Rust CLI writes output_hash = BLAKE3(WASM bytes) into cook-receipt.json
 * and pushes it to game_receipts. The browser Playwright spec (tps-dflss.spec.ts)
 * also reads the WASM file and embeds the same hash. This endpoint lets you:
 *   1. Query all receipts that share an output_hash — cook receipt + game receipts.
 *   2. Confirm that the binary served to the browser is the one that was cooked.
 *   3. Detect binary substitution: if a game receipt has a different output_hash
 *      than the cook receipt for the same session, a different WASM was served.
 *
 * Returns:
 *   {
 *     output_hash: string,
 *     receipts: ReceiptSummary[],   // all rows with this output_hash
 *     cross_check: {
 *       cook_receipts: number,       // rows where milestone contains 'Cook'
 *       game_receipts: number,       // rows where milestone contains 'Game' or 'Session'
 *       verdict: 'MATCH' | 'MISMATCH' | 'COOK_ONLY' | 'GAME_ONLY' | 'NO_DATA',
 *     }
 *   }
 *
 * MISMATCH means two receipts for the same session have different output_hashes
 * (binary substitution detected). COOK_ONLY / GAME_ONLY means only one side
 * of the pipeline has been run yet.
 */
import { createClient } from '@supabase/supabase-js';

interface ReceiptSummary {
  id: string;
  milestone: string;
  verdict: string;
  engine_source: string;
  output_hash: string;
  proven_at: string;
  session_id: string | null;
}

export default defineEventHandler(async (event) => {
  const query = getQuery(event);
  const outputHash = typeof query.output_hash === 'string' ? query.output_hash.trim() : null;

  if (!outputHash) {
    throw createError({ statusCode: 400, statusMessage: 'output_hash query param required' });
  }

  // Validate: BLAKE3 hex is 64 lowercase hex chars
  if (!/^[0-9a-f]{64}$/.test(outputHash)) {
    throw createError({
      statusCode: 400,
      statusMessage: 'output_hash must be 64 lowercase hex chars (BLAKE3 of WASM bytes)',
    });
  }

  const config = useRuntimeConfig(event);
  const supabaseUrl = config.public.supabaseUrl as string;
  const serviceKey = (config.supabaseServiceRoleKey as string) || (config.public.supabaseAnonKey as string);

  if (!supabaseUrl || !serviceKey) {
    throw createError({ statusCode: 503, statusMessage: 'Supabase not configured' });
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const sb = createClient<any>(supabaseUrl, serviceKey);

  const { data, error } = await sb
    .from('game_receipts')
    .select('id, milestone, verdict, engine_source, output_hash, proven_at, session_id')
    .eq('output_hash', outputHash)
    .order('proven_at', { ascending: true });

  if (error) throw createError({ statusCode: 500, statusMessage: error.message });

  const receipts = (data ?? []) as ReceiptSummary[];

  // Classify each receipt as cook vs game by milestone name
  const isCookReceipt = (r: ReceiptSummary) =>
    /cook|html5|verify|package/i.test(r.milestone);
  const isGameReceipt = (r: ReceiptSummary) =>
    /game|session|proof|tps|dflss/i.test(r.milestone);

  const cookCount = receipts.filter(isCookReceipt).length;
  const gameCount = receipts.filter(isGameReceipt).length;

  let verdict: 'MATCH' | 'MISMATCH' | 'COOK_ONLY' | 'GAME_ONLY' | 'NO_DATA';

  if (receipts.length === 0) {
    verdict = 'NO_DATA';
  } else if (cookCount > 0 && gameCount > 0) {
    // Both sides have receipts with the same output_hash → binary matches
    verdict = 'MATCH';
  } else if (cookCount > 0) {
    verdict = 'COOK_ONLY';
  } else if (gameCount > 0) {
    verdict = 'GAME_ONLY';
  } else {
    // Receipts exist but none classified as cook or game (unusual milestone names)
    verdict = 'MATCH';
  }

  return {
    output_hash: outputHash,
    receipts,
    cross_check: {
      cook_receipts: cookCount,
      game_receipts: gameCount,
      total: receipts.length,
      verdict,
    },
  };
});
