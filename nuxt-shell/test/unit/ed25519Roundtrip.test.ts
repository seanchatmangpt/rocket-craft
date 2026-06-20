import { describe, it, expect } from 'vitest';
import * as ed from '@noble/ed25519';
import { canonicalReceiptPayload, verifyReceiptSignature, type ReceiptSigFields } from '../../server/utils/ed25519';

/**
 * Ed25519 sign→verify roundtrip — guards the Rust↔Nuxt canonical-payload contract.
 *
 * The Rust CLI (rocket-sdk/src/signing.rs::receipt_signing_payload) signs:
 *   {"proven_at":<j>,"receipt_hash":<j>,"session_id":<j|null>,"verdict":<j>}
 * with sorted keys. If the Nuxt canonical payload ever drifts from this, every
 * real signed cook receipt would be rejected 401 (the bug this test prevents).
 */

// Replicates signing.rs::receipt_signing_payload EXACTLY (independent of our util,
// so the test fails if EITHER side drifts).
function rustCanonicalPayload(f: ReceiptSigFields): string {
  const j = (v: unknown) => JSON.stringify(v);
  return `{"proven_at":${j(f.proven_at)},"receipt_hash":${j(f.receipt_hash)},"session_id":${j(f.session_id)},"verdict":${j(f.verdict)}}`;
}

const CASES: ReceiptSigFields[] = [
  { proven_at: '2026-06-20T00:00:00Z', receipt_hash: 'blake3:abc123', session_id: 'sess-1', verdict: 'PASS' },
  { proven_at: '2026-06-20T01:02:03.456Z', receipt_hash: 'a'.repeat(64), session_id: null, verdict: 'FAIL' },
  { proven_at: '2026-06-20T09:09:09Z', receipt_hash: 'deadbeef', session_id: '0904ff08-a4ec-4d03-83b9-309adc3d5aa9', verdict: 'PROVEN' },
];

describe('ed25519 receipt signing contract', () => {
  it('Nuxt canonical payload byte-matches the Rust signing.rs format', () => {
    for (const c of CASES) {
      expect(canonicalReceiptPayload(c)).toBe(rustCanonicalPayload(c));
    }
  });

  it('session_id absent normalises to null (matches serde Option<&str>)', () => {
    const withNull = canonicalReceiptPayload({ proven_at: 't', receipt_hash: 'h', session_id: null, verdict: 'PASS' });
    expect(withNull).toContain('"session_id":null');
  });

  it('keys are sorted: proven_at, receipt_hash, session_id, verdict', () => {
    const s = canonicalReceiptPayload(CASES[0]!);
    expect(s.indexOf('proven_at')).toBeLessThan(s.indexOf('receipt_hash'));
    expect(s.indexOf('receipt_hash')).toBeLessThan(s.indexOf('session_id'));
    expect(s.indexOf('session_id')).toBeLessThan(s.indexOf('verdict'));
  });

  it('full roundtrip: sign the canonical payload, verifyReceiptSignature accepts it', async () => {
    for (const c of CASES) {
      const priv = ed.utils.randomPrivateKey();
      const pub = await ed.getPublicKeyAsync(priv);
      const msg = new TextEncoder().encode(canonicalReceiptPayload(c));
      const sig = await ed.signAsync(msg, priv);
      const sigB64 = Buffer.from(sig).toString('base64');
      const pubB64 = Buffer.from(pub).toString('base64');
      expect(await verifyReceiptSignature(c, sigB64, pubB64)).toBe(true);
    }
  });

  it('rejects a signature over a tampered field (wrong verdict)', async () => {
    const c = CASES[0]!;
    const priv = ed.utils.randomPrivateKey();
    const pub = await ed.getPublicKeyAsync(priv);
    const msg = new TextEncoder().encode(canonicalReceiptPayload(c));
    const sig = await ed.signAsync(msg, priv);
    const sigB64 = Buffer.from(sig).toString('base64');
    const pubB64 = Buffer.from(pub).toString('base64');
    // Verify with verdict flipped — must fail.
    expect(await verifyReceiptSignature({ ...c, verdict: 'FAIL' }, sigB64, pubB64)).toBe(false);
  });

  it('returns false (never throws) on malformed base64', async () => {
    expect(await verifyReceiptSignature(CASES[0]!, '!!!notb64', '!!!notb64')).toBe(false);
  });
});
