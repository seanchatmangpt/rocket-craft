/**
 * GET /api/game/wasm-verify?session_id=<uuid>&path=<wasm path>
 *
 * Independent, server-side WASM tamper detection. Unlike wasm-crosscheck (which
 * compares output_hash values already stored across receipts), this endpoint
 * RE-HASHES the actual binary on disk and compares it to the cook-receipt's
 * output_hash. It trusts no embedded claim — it recomputes the evidence.
 *
 * This closes the cook-to-game tamper gap: if the served Brm.wasm is swapped
 * after the cook (different binary than the one proven), the recomputed BLAKE3
 * will not match the cook receipt's output_hash → MISMATCH.
 *
 * Query:
 *   session_id    optional — look up the cook receipt's output_hash for this session
 *   expected_hash optional — compare against this hash directly (overrides DB lookup)
 *   path          optional — wasm file path (default WASM_ARCHIVE_DIR/Brm.wasm or
 *                            /private/tmp/brm-html5-archive/HTML5/Brm.wasm)
 *
 * Returns:
 *   { wasm_path, size_bytes, computed_hash, expected_hash, verdict, source }
 *   verdict: MATCH | MISMATCH | NO_EXPECTED_HASH | NO_WASM
 *
 * BLAKE3(175 MB) ≈ 2.8 s; result is cached by (path, size, mtime) so repeated
 * calls are O(1) until the binary changes.
 */

import { createClient } from '@supabase/supabase-js';
import { blake3 } from '@noble/hashes/blake3.js';
import { readFileSync, statSync } from 'node:fs';

interface HashCacheEntry { hash: string; size: number; mtimeMs: number }
const hashCache = new Map<string, HashCacheEntry>();

function hashWasm(path: string): { hash: string; size: number } | null {
  let st: ReturnType<typeof statSync>;
  try {
    st = statSync(path);
  } catch {
    return null;
  }
  const cached = hashCache.get(path);
  if (cached && cached.size === st.size && cached.mtimeMs === st.mtimeMs) {
    return { hash: cached.hash, size: cached.size };
  }
  const bytes = readFileSync(path);
  const hash = Buffer.from(blake3(bytes)).toString('hex');
  hashCache.set(path, { hash, size: st.size, mtimeMs: st.mtimeMs });
  return { hash, size: st.size };
}

export default defineEventHandler(async (event) => {
  const query = getQuery(event);
  const sessionId = typeof query.session_id === 'string' ? query.session_id : null;
  const explicitHash = typeof query.expected_hash === 'string' ? query.expected_hash : null;

  const defaultDir = process.env.WASM_ARCHIVE_DIR || '/private/tmp/brm-html5-archive/HTML5';
  const wasmPath = typeof query.path === 'string' ? query.path : `${defaultDir}/Brm.wasm`;

  // 1. Re-hash the actual binary on disk.
  const hashed = hashWasm(wasmPath);
  if (!hashed) {
    return {
      wasm_path: wasmPath,
      size_bytes: 0,
      computed_hash: null,
      expected_hash: explicitHash,
      verdict: 'NO_WASM',
      source: 'disk',
    };
  }

  // 2. Resolve the expected hash: explicit query param, else latest cook receipt.
  let expectedHash = explicitHash;
  let source = explicitHash ? 'query' : 'none';
  if (!expectedHash) {
    const config = useRuntimeConfig(event);
    const supabaseUrl = (config.public.supabaseUrl as string) || 'http://localhost:54321';
    const serviceKey = config.supabaseServiceRoleKey as string;
    if (serviceKey) {
      const sb = createClient<any>(supabaseUrl, serviceKey);
      let q = sb
        .from('game_receipts')
        .select('output_hash, session_id, proven_at')
        .not('output_hash', 'is', null)
        .order('proven_at', { ascending: false })
        .limit(1);
      if (sessionId) q = q.eq('session_id', sessionId);
      const { data } = await q.maybeSingle();
      if (data?.output_hash) {
        expectedHash = data.output_hash as string;
        source = 'cook_receipt';
      }
    }
  }

  // 3. Compare.
  let verdict: 'MATCH' | 'MISMATCH' | 'NO_EXPECTED_HASH';
  if (!expectedHash) {
    verdict = 'NO_EXPECTED_HASH';
  } else {
    verdict = hashed.hash === expectedHash ? 'MATCH' : 'MISMATCH';
  }

  return {
    wasm_path: wasmPath,
    size_bytes: hashed.size,
    computed_hash: hashed.hash,
    expected_hash: expectedHash,
    verdict,
    source,
  };
});
