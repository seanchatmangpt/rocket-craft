import { describe, it, expect } from 'vitest';
import { blake3 } from '@noble/hashes/blake3.js';
import { canonicalOcelEventHash, type OcelEventHashFields } from '../../app/utils/ocelEventHash';

/**
 * Cross-implementation OCEL event-hash contract.
 *
 * The browser persistence path (useGameSessionPersistence → canonicalOcelEventHash)
 * MUST produce the same event_hash the server recomputes (session-replay.get.ts) and
 * the server writes (session-seed.post.ts). If they diverge, real browser sessions
 * replay as hash_convergent=false — a false tamper alarm — and the chain proof fails.
 *
 * This test independently replicates the SERVER canonicalize() and asserts the
 * browser util byte-matches it, so neither side can drift unnoticed.
 */

function blake3Hex(input: string): string {
  return Array.from(blake3(new TextEncoder().encode(input)))
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
}

// Verbatim replica of server canonicalize() in session-seed/session-replay:
//   JSON.stringify(obj, Object.keys(obj).sort())
function serverEventHash(f: OcelEventHashFields): string {
  const obj: Record<string, unknown> = {
    session_id: f.session_id,
    activity: f.activity,
    timestamp_ms: f.timestamp_ms,
    prev_hash: f.prev_hash,
    attributes: f.attributes,
  };
  return blake3Hex(JSON.stringify(obj, Object.keys(obj).sort()));
}

const CASES: OcelEventHashFields[] = [
  { session_id: 'sess-1', activity: 'GameSessionStarted', timestamp_ms: 1781000000000, prev_hash: null, attributes: {} },
  { session_id: 'sess-1', activity: 'FrameRendered', timestamp_ms: 1781000000200, prev_hash: 'a'.repeat(64), attributes: { source: 'ue4_raf' } },
  { session_id: '0904ff08-a4ec-4d03-83b9-309adc3d5aa9', activity: 'InputAdmitted', timestamp_ms: 1781000000400, prev_hash: 'b'.repeat(64), attributes: { intent_type: 'Interact', seq: 2 } },
];

describe('OCEL event hash cross-implementation contract', () => {
  it('browser canonicalOcelEventHash byte-matches the server canonicalize formula', () => {
    for (const c of CASES) {
      expect(canonicalOcelEventHash(c), JSON.stringify(c)).toBe(serverEventHash(c));
    }
  });

  it('produces a 64-char lowercase hex BLAKE3 string', () => {
    expect(canonicalOcelEventHash(CASES[0]!)).toMatch(/^[0-9a-f]{64}$/);
  });

  it('is deterministic and order-independent on input field order', () => {
    const a = canonicalOcelEventHash({ session_id: 's', activity: 'X', timestamp_ms: 1, prev_hash: null, attributes: { k: 1 } });
    const b = canonicalOcelEventHash({ attributes: { k: 1 }, prev_hash: null, timestamp_ms: 1, activity: 'X', session_id: 's' } as OcelEventHashFields);
    expect(a).toBe(b);
  });

  it('changing a hashed field changes the hash (prev_hash chains)', () => {
    const base = canonicalOcelEventHash(CASES[0]!);
    const chained = canonicalOcelEventHash({ ...CASES[0]!, prev_hash: 'c'.repeat(64) });
    expect(chained).not.toBe(base);
  });
});
