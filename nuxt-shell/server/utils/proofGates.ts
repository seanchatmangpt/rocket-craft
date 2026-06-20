/**
 * server/utils/proofGates.ts
 *
 * Pure proof gate functions extracted from cook-receipt.post.ts.
 * No Nitro globals — fully unit-testable.
 *
 * Pipes-and-filters pattern: each gate is an independent pure validator.
 * Gates run in order; first failure short-circuits the chain.
 */

export type GateResult =
  | { ok: true }
  | { ok: false; statusCode: number; message: string };

export const DECLARED_LIFECYCLE = ['GameSessionStarted', 'FrameRendered', 'InputAdmitted'] as const;

// ── Gate 1: receipt_hash format ─────────────────────────────────────────────

export function gateReceiptHashFormat(receiptHash: unknown): GateResult {
  if (typeof receiptHash !== 'string') {
    return { ok: false, statusCode: 400, message: 'receipt_hash must be 64 hex chars (BLAKE3)' };
  }
  if (receiptHash.length !== 64 || !/^[0-9a-f]+$/.test(receiptHash)) {
    return { ok: false, statusCode: 400, message: 'receipt_hash must be 64 hex chars (BLAKE3)' };
  }
  return { ok: true };
}

// ── Gate 2: engine_source not synthetic ─────────────────────────────────────

export function gateNotSynthetic(engineSource: unknown): GateResult {
  if (engineSource === 'synthetic') {
    return { ok: false, statusCode: 422, message: 'engine_source: synthetic is rejected by the proof gate' };
  }
  return { ok: true };
}

// ── Gate 3: minimum lifecycle activities present ─────────────────────────────

export function gateLifecycleComplete(ocelLifecycle: unknown): GateResult {
  const lifecycle: string[] = Array.isArray(ocelLifecycle) ? ocelLifecycle : [];
  const missing = DECLARED_LIFECYCLE.filter(a => !lifecycle.includes(a));
  if (missing.length > 0) {
    return {
      ok: false,
      statusCode: 422,
      message: `ocel_lifecycle missing required activities: ${missing.join(', ')}`,
    };
  }
  return { ok: true };
}

// ── Gate 4: required fields present ─────────────────────────────────────────

export function gateRequiredFields(body: {
  verdict?: unknown;
  milestone?: unknown;
  engine_source?: unknown;
  receipt_hash?: unknown;
}): GateResult {
  if (!body.verdict || !body.milestone || !body.engine_source || !body.receipt_hash) {
    return {
      ok: false,
      statusCode: 400,
      message: 'Missing required fields: verdict, milestone, engine_source, receipt_hash',
    };
  }
  return { ok: true };
}

// ── Composed gate chain ──────────────────────────────────────────────────────

export interface ProofGateInput {
  verdict?: unknown;
  milestone?: unknown;
  engine_source?: unknown;
  receipt_hash?: unknown;
  ocel_lifecycle?: unknown;
}

/**
 * Run all 3 stateless proof gates in order (gate 4/Ed25519 is skipped — it
 * requires async crypto and the pubkey env var, tested separately).
 * Returns the first failure or { ok: true } if all pass.
 */
export function runProofGates(input: ProofGateInput): GateResult {
  const g1 = gateRequiredFields(input);
  if (!g1.ok) return g1;

  const g2 = gateReceiptHashFormat(input.receipt_hash);
  if (!g2.ok) return g2;

  const g3 = gateNotSynthetic(input.engine_source);
  if (!g3.ok) return g3;

  const g4 = gateLifecycleComplete(input.ocel_lifecycle);
  if (!g4.ok) return g4;

  return { ok: true };
}
