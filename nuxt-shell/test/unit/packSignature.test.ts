// @vitest-environment node
import { describe, it, expect } from 'vitest';
import * as ed from '@noble/ed25519';
import { signEd25519, verifyEd25519 } from '../../server/utils/ed25519';

/**
 * Evidence-pack authentication: signEd25519 signs the pack_hash with the active
 * Ed25519 key so a verifier can prove ORIGIN. This guards the sign→verify
 * roundtrip and the graceful behaviour when no key is configured.
 */

function b64(bytes: Uint8Array): string {
  let s = '';
  for (const x of bytes) s += String.fromCharCode(x);
  return btoa(s);
}

describe('evidence-pack signature (signEd25519 / verifyEd25519)', () => {
  it('signs a pack_hash and verifies against the matching public key', async () => {
    const priv = ed.utils.randomPrivateKey();
    const pub = await ed.getPublicKeyAsync(priv);
    const packHash = 'a'.repeat(64); // BLAKE3 hex of a pack

    const sig = await signEd25519(packHash, b64(priv));
    expect(sig).toMatch(/^[A-Za-z0-9+/]+=*$/); // base64
    expect(await verifyEd25519(packHash, sig!, b64(pub))).toBe(true);
  });

  it('rejects a signature against the WRONG public key (origin proof)', async () => {
    const priv = ed.utils.randomPrivateKey();
    const otherPub = await ed.getPublicKeyAsync(ed.utils.randomPrivateKey());
    const sig = await signEd25519('b'.repeat(64), b64(priv));
    expect(await verifyEd25519('b'.repeat(64), sig!, b64(otherPub))).toBe(false);
  });

  it('rejects a signature over a TAMPERED pack_hash', async () => {
    const priv = ed.utils.randomPrivateKey();
    const pub = await ed.getPublicKeyAsync(priv);
    const sig = await signEd25519('c'.repeat(64), b64(priv));
    // Same key, different message → must not verify.
    expect(await verifyEd25519('d'.repeat(64), sig!, b64(pub))).toBe(false);
  });

  it('returns null (graceful, unsigned pack) when no signing key is configured', async () => {
    expect(await signEd25519('e'.repeat(64), undefined)).toBeNull();
    expect(await signEd25519('e'.repeat(64), '')).toBeNull();
  });

  it('returns null (never throws) on a malformed private key', async () => {
    expect(await signEd25519('f'.repeat(64), '!!!not-base64!!!')).toBeNull();
  });
});
