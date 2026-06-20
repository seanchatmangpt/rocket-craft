import { blake3 } from '@noble/hashes/blake3.js';

/**
 * Canonical OCEL event hash — MUST byte-match the server formula used by
 * session-seed.post.ts and session-replay.get.ts (and therefore what the chain
 * proof verifies). The browser persistence path previously hashed a different
 * shape ({id,timestamp,type,data,prev_hash} via useHashChain), so real browser
 * sessions replayed as hash_convergent=false — a false tamper alarm.
 *
 * The canonical payload is sorted-key JSON over exactly:
 *   { activity, attributes, prev_hash, session_id, timestamp_ms }
 *
 * NOTE: the server uses `JSON.stringify(obj, Object.keys(obj).sort())`. The array
 * replacer is applied at ALL nesting levels, so nested `attributes` keys (not in
 * the top-level key set) are dropped from the serialization. We replicate that
 * EXACT call so the hash matches — do not "improve" it to a recursive canonical
 * or it will diverge from the server.
 */
export interface OcelEventHashFields {
  session_id: string;
  activity: string;
  timestamp_ms: number;
  prev_hash: string | null;
  attributes: Record<string, unknown>;
}

function blake3Hex(input: string): string {
  return Array.from(blake3(new TextEncoder().encode(input)))
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
}

export function canonicalOcelEventHash(fields: OcelEventHashFields): string {
  const obj: Record<string, unknown> = {
    session_id: fields.session_id,
    activity: fields.activity,
    timestamp_ms: fields.timestamp_ms,
    prev_hash: fields.prev_hash,
    attributes: fields.attributes,
  };
  // Identical construction to server canonicalize() — sorted top-level keys as the
  // JSON.stringify replacer array.
  const payload = JSON.stringify(obj, Object.keys(obj).sort());
  return blake3Hex(payload);
}
