/**
 * Ed25519 receipt signing/verification — the SINGLE source of truth for the
 * canonical payload that the Rust CLI (rocket-sdk/src/signing.rs) signs and the
 * Nuxt proof gate verifies.
 *
 * The canonical payload is sorted-key JSON over EXACTLY four fields:
 *   {"proven_at":<iso>,"receipt_hash":<hash>,"session_id":<uuid|null>,"verdict":<str>}
 *
 * This MUST byte-match signing.rs::receipt_signing_payload. Previously cook-receipt
 * verified the entire request body while the Rust CLI signed only these four fields,
 * so every real signed receipt was rejected 401. Consolidating here prevents that
 * divergence from ever recurring.
 */
import * as ed from '@noble/ed25519';

export interface ReceiptSigFields {
  proven_at: string;
  receipt_hash: string;
  session_id: string | null;
  verdict: string;
}

/**
 * Build the canonical signing payload — sorted keys, exactly the four signed
 * fields. session_id is normalised to null when absent (matches serde's
 * Option<&str> → null).
 */
export function canonicalReceiptPayload(fields: ReceiptSigFields): string {
  // Keys are already alphabetical: proven_at < receipt_hash < session_id < verdict.
  return JSON.stringify({
    proven_at: fields.proven_at,
    receipt_hash: fields.receipt_hash,
    session_id: fields.session_id ?? null,
    verdict: fields.verdict,
  });
}

function b64ToBytes(b64: string): Uint8Array {
  return Uint8Array.from(atob(b64), (c) => c.charCodeAt(0));
}

/**
 * Verify an Ed25519 signature (base64) over the canonical receipt payload.
 * Returns false on any malformed input rather than throwing.
 */
export async function verifyReceiptSignature(
  fields: ReceiptSigFields,
  sigB64: string,
  pubKeyB64: string,
): Promise<boolean> {
  try {
    const message = new TextEncoder().encode(canonicalReceiptPayload(fields));
    return await ed.verifyAsync(b64ToBytes(sigB64), message, b64ToBytes(pubKeyB64));
  } catch {
    return false;
  }
}
