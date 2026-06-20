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
 *   {"proven_at":<iso>, "receipt_hash":<blake3:hex>, "session_id":<uuid|null>, "verdict":<PASS|FAIL>}
 * This must match signing.rs::receipt_signing_payload exactly — enforced by the
 * shared server/utils/ed25519.ts (single source of truth).
 */

import { verifyReceiptSignature } from '../../utils/ed25519';

interface VerifyBody {
  verdict: string;
  receipt_hash: string;
  session_id: string | null;
  proven_at: string;
  ed25519_sig: string;
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

  // Shared canonical payload + verification (single source of truth with cook-receipt
  // and signing.rs) — see server/utils/ed25519.ts.
  const verified = await verifyReceiptSignature(
    {
      proven_at: body.proven_at,
      receipt_hash: body.receipt_hash,
      session_id: body.session_id ?? null,
      verdict: body.verdict,
    },
    body.ed25519_sig,
    pubKeyB64,
  );

  return { verified, algorithm: 'Ed25519' };
});
