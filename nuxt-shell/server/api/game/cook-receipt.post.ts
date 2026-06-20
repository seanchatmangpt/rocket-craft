/**
 * POST /api/game/cook-receipt
 *
 * Server-side gate for Rust CLI cook pipeline receipts.
 * The rocket-cmd HTML5 pipeline calls this instead of writing directly to
 * Supabase — ensuring every cook receipt passes Ed25519 signature validation
 * before it enters the game_receipts table.
 *
 * Flow:
 *   1. Verify Ed25519 signature against ROCKET_SIGNING_PUBKEY
 *   2. Validate engine_source ≠ 'synthetic' (proof gate)
 *   3. Validate ocel_lifecycle contains the declared minimum lifecycle
 *   4. Insert into game_receipts with ed25519_sig column
 *
 * Body: {
 *   session_id: string | null
 *   verdict: 'PASS' | 'FAIL'
 *   milestone: string
 *   engine_source: string       — must NOT be 'synthetic'
 *   ocel_lifecycle: string[]    — must include DECLARED_LIFECYCLE activities
 *   ocel_event_count: number
 *   receipt_hash: string        — 64-char BLAKE3 hex
 *   output_hash?: string        — BLAKE3 of WASM binary (optional)
 *   proven_at: string           — ISO timestamp
 *   payload: Record<string,unknown>
 *   ed25519_sig?: string        — base64 Ed25519 signature over canonical JSON of body
 * }
 *
 * Returns: { receipt_id, verdict, chain_verified }
 */

import { createClient } from '@supabase/supabase-js'
import * as ed from '@noble/ed25519'
import { emitOtelSpans } from '../../utils/otlp-emitter'
import { runProofGates } from '../../utils/proofGates'
import { auditGateRun } from '../../utils/proofGateAudit'

function canonicalJSON(obj: unknown): string {
  if (obj === null || obj === undefined) return 'null'
  if (typeof obj === 'number' || typeof obj === 'boolean') return JSON.stringify(obj)
  if (typeof obj === 'string') return JSON.stringify(obj)
  if (Array.isArray(obj)) return `[${obj.map(canonicalJSON).join(',')}]`
  const o = obj as Record<string, unknown>
  const keys = Object.keys(o).sort()
  return `{${keys.map(k => `${JSON.stringify(k)}:${canonicalJSON(o[k])}`).join(',')}}`
}

async function verifyEd25519(
  payload: unknown,
  sigB64: string,
  pubKeyB64: string,
): Promise<boolean> {
  try {
    const sigBytes = Uint8Array.from(atob(sigB64), c => c.charCodeAt(0))
    const pubBytes = Uint8Array.from(atob(pubKeyB64), c => c.charCodeAt(0))
    const message = new TextEncoder().encode(canonicalJSON(payload))
    return await ed.verifyAsync(sigBytes, message, pubBytes)
  } catch {
    return false
  }
}

export default defineEventHandler(async (event) => {
  const body = await readBody(event)
  const {
    session_id,
    verdict,
    milestone,
    engine_source,
    ocel_lifecycle,
    ocel_event_count,
    receipt_hash,
    output_hash,
    proven_at,
    payload,
    ed25519_sig,
  } = body ?? {}

  // ── Proof gates (pure functions from proofGates.ts) ───────────────────────
  const gateInput = { verdict, milestone, engine_source, receipt_hash, ocel_lifecycle }
  const gateResult = runProofGates(gateInput)

  // Supabase client needed for audit; construct lazily but before throwing
  const supabaseUrl = process.env.SUPABASE_URL ?? 'http://localhost:54321'
  const serviceKey = process.env.SUPABASE_SERVICE_ROLE_KEY
  const sbForAudit = serviceKey ? createClient(supabaseUrl, serviceKey) : null

  if (!gateResult.ok) {
    // Map message → gate name for the audit row
    const gateNameMap: Record<string, string> = {
      'Missing required fields': 'required_fields',
      'receipt_hash must be': 'receipt_hash_format',
      'synthetic': 'not_synthetic',
      'ocel_lifecycle': 'lifecycle_complete',
    }
    const failedGate = Object.entries(gateNameMap).find(([k]) => gateResult.message?.includes(k))?.[1] ?? 'unknown'
    if (sbForAudit && session_id) {
      auditGateRun(sbForAudit, session_id, gateInput, failedGate, gateResult.message ?? null)
    }
    throw createError({ statusCode: gateResult.statusCode ?? 400, message: gateResult.message ?? 'Proof gate rejected' })
  }

  // Gates passed — record audit (fire-and-forget)
  if (sbForAudit && session_id) {
    auditGateRun(sbForAudit, session_id, gateInput, null, null)
  }

  // ── Proof gate 4: Ed25519 signature (required in production) ───────────────
  const pubKeyB64 = process.env.ROCKET_SIGNING_PUBKEY
  if (pubKeyB64) {
    if (!ed25519_sig) {
      throw createError({ statusCode: 401, message: 'ed25519_sig required: ROCKET_SIGNING_PUBKEY is configured' })
    }
    const { ed25519_sig: _sig, ...sigPayload } = body
    const valid = await verifyEd25519(sigPayload, ed25519_sig, pubKeyB64)
    if (!valid) {
      throw createError({ statusCode: 401, message: 'Ed25519 signature verification failed' })
    }
  }

  // ── Insert into game_receipts ───────────────────────────────────────────────
  if (!serviceKey) {
    throw createError({ statusCode: 503, message: 'SUPABASE_SERVICE_ROLE_KEY not configured' })
  }

  const supabase = sbForAudit!

  const { data: receipt, error: insertErr } = await supabase
    .from('game_receipts')
    .insert({
      session_id: session_id ?? null,
      verdict,
      milestone,
      engine_source,
      ocel_lifecycle: Array.isArray(ocel_lifecycle) ? ocel_lifecycle : [],
      ocel_event_count: Number(ocel_event_count ?? 0),
      receipt_hash,
      output_hash: output_hash ?? null,
      proven_at: proven_at ?? new Date().toISOString(),
      payload: payload ?? {},
      ed25519_sig: ed25519_sig ?? null,
    })
    .select('id, verdict, session_id')
    .single()

  if (insertErr) {
    throw createError({ statusCode: 500, message: `Failed to insert cook receipt: ${insertErr.message}` })
  }

  // ── Chain verify (if session_id provided) ──────────────────────────────────
  let chainVerified = false
  if (session_id) {
    const { data: chainResult } = await supabase
      .rpc('verify_event_chain', { p_session_id: session_id })
    chainVerified = chainResult === true
  }

  // ── receipt.emit OTel span (truex LIVE-13 pattern) ────────────────────────
  // Emit a span marking this receipt was witnessed by the proof aggregator.
  // Goes to OTel (not ocel_events) to avoid breaking the sequential BLAKE3
  // chain — ReceiptEmitted is a meta/monitoring event, not a gameplay event.
  if (session_id && receipt.verdict === 'PASS') {
    emitOtelSpans([{
      activity: 'ReceiptEmitted',
      timestamp_ms: Date.now(),
      session_id,
      // OCEL linkage fields — close the OTel ↔ OCEL correlation gap
      receipt_id: receipt.id,
      receipt_hash,
      ocel_event_count: Number(ocel_event_count ?? 0),
      attributes: {
        signer: 'proof_aggregator',
        verdict,
        algorithm: 'BLAKE3',
      },
    }]).catch(() => {})
  }

  return {
    receipt_id: receipt.id,
    verdict: receipt.verdict,
    session_id: receipt.session_id,
    chain_verified: chainVerified,
    engine_source,
    proof_gates_passed: ['receipt_hash_format', 'not_synthetic', 'lifecycle_complete', ...(pubKeyB64 ? ['ed25519_sig'] : [])],
  }
})
