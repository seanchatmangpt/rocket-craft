/**
 * POST /api/game/verify-signature
 *
 * Verifies an Ed25519 receipt signature using the ROCKET_SIGNING_PUBKEY env var.
 * Pattern: ~/dashboard.bak/server/api/verify-receipt.post.ts (Ed25519 + canonical JSON)
 *
 * Body: { receipt_id, verdict, receipt_hash, session_id, proven_at, ed25519_sig }
 * Returns: { verified: boolean, algorithm: "Ed25519", error?: string }
 *
 * The signing payload is canonical sorted-key JSON:
 *   {"proven_at":<iso>, "receipt_hash":<sha256:hex>, "session_id":<uuid|null>, "verdict":<PASS|FAIL>}
 * This must match signing.rs::receipt_signing_payload exactly.
 */

interface VerifyBody {
  verdict: string;
  receipt_hash: string;
  session_id: string | null;
  proven_at: string;
  ed25519_sig: string;
}

function canonicalPayload(body: VerifyBody): string {
  // Sorted-key JSON — matches signing.rs::receipt_signing_payload
  return JSON.stringify({
    proven_at: body.proven_at,
    receipt_hash: body.receipt_hash,
    session_id: body.session_id,
    verdict: body.verdict,
  }, Object.keys({ proven_at: 1, receipt_hash: 1, session_id: 1, verdict: 1 }).sort());
}

function b64ToBuffer(b64: string): ArrayBuffer {
  const bin = atob(b64);
  const buf = new ArrayBuffer(bin.length);
  const view = new Uint8Array(buf);
  for (let i = 0; i < bin.length; i++) view[i] = bin.charCodeAt(i);
  return buf;
}

async function verifyEd25519(publicKeyB64: string, message: Uint8Array, sigB64: string): Promise<boolean> {
  try {
    const pubKeyBuf = b64ToBuffer(publicKeyB64);
    const sigBuf = b64ToBuffer(sigB64);
    const pubKey = await crypto.subtle.importKey('raw', pubKeyBuf, { name: 'Ed25519' }, false, ['verify']);
    return await crypto.subtle.verify('Ed25519', pubKey, sigBuf, message.buffer as ArrayBuffer);
  } catch {
    return false;
  }
}

export default defineEventHandler(async (event) => {
  const body = await readBody<VerifyBody>(event);

  if (!body?.ed25519_sig || !body.verdict || !body.receipt_hash || !body.proven_at) {
    throw createError({ statusCode: 400, statusMessage: 'verdict, receipt_hash, proven_at, ed25519_sig required' });
  }

  const config = useRuntimeConfig(event);
  const pubKeyB64 = config.rocketSigningPubkey as string | undefined;

  if (!pubKeyB64) {
    return { verified: false, algorithm: 'Ed25519', error: 'ROCKET_SIGNING_PUBKEY not configured' };
  }

  const payload = canonicalPayload(body);
  const msgBytes = new TextEncoder().encode(payload);
  const verified = await verifyEd25519(pubKeyB64, msgBytes, body.ed25519_sig);

  return { verified, algorithm: 'Ed25519' };
});
