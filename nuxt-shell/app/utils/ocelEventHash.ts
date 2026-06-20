import { blake3 } from '@noble/hashes/blake3.js';

/**
 * Canonical OCEL event hash — MUST byte-match the server formula used by
 * session-seed.post.ts and session-replay.get.ts (and therefore what the chain
 * proof verifies). The browser persistence path previously hashed a different
 * shape ({id,timestamp,type,data,prev_hash} via useHashChain), so real browser
 * sessions replayed as hash_convergent=false — a false tamper alarm.
 *
 * The canonical payload is RECURSIVE sorted-key JSON over exactly:
 *   { activity, attributes, prev_hash, session_id, timestamp_ms }
 * byte-matching the server (session-seed/session-replay canonicalJSON) and the
 * Rust CLI (canonical_json). Recursive so nested `attributes` are actually covered
 * by the hash and all four implementations converge.
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

/** Recursive canonical JSON — byte-matches server canonicalJSON + Rust canonical_json. */
function canonicalJSON(value: unknown): string {
  if (value === null || value === undefined) return 'null';
  if (typeof value === 'number' || typeof value === 'boolean' || typeof value === 'string') {
    return JSON.stringify(value);
  }
  if (Array.isArray(value)) return `[${value.map(canonicalJSON).join(',')}]`;
  const obj = value as Record<string, unknown>;
  return `{${Object.keys(obj).sort().map((k) => `${JSON.stringify(k)}:${canonicalJSON(obj[k])}`).join(',')}}`;
}

export function canonicalOcelEventHash(fields: OcelEventHashFields): string {
  return blake3Hex(canonicalJSON({
    session_id: fields.session_id,
    activity: fields.activity,
    timestamp_ms: fields.timestamp_ms,
    prev_hash: fields.prev_hash,
    attributes: fields.attributes,
  }));
}
