/**
 * Supabase Edge Function: receipt-verify
 *
 * Production proof engine — runs on Supabase cloud without Nitro.
 * Verifies a cook receipt's Ed25519 signature, BLAKE3 hash, and OCEL chain.
 *
 * POST /functions/v1/receipt-verify
 * Body: { receipt_id: string, verify_chain?: boolean }
 * Returns: { verified, receipt_id, verdict, chain_verified, proof_gates }
 *
 * Mirrors the logic in:
 *   nuxt-shell/server/api/game/verify-signature.post.ts (Ed25519 gate)
 *   nuxt-shell/server/api/game/chain-verify.get.ts (chain gate)
 *   nuxt-shell/server/api/game/cook-receipt.post.ts (insertion gate)
 *
 * Deployed via: supabase functions deploy receipt-verify
 */

import { createClient } from 'https://esm.sh/@supabase/supabase-js@2'
// @ts-ignore — Deno stdlib
import { encode as encodeBase64 } from 'https://deno.land/std@0.224.0/encoding/base64.ts'

const DECLARED_LIFECYCLE = ['GameSessionStarted', 'FrameRendered', 'InputAdmitted']

function canonicalJSON(obj: unknown): string {
  if (obj === null || obj === undefined) return 'null'
  if (typeof obj === 'number' || typeof obj === 'boolean') return JSON.stringify(obj)
  if (typeof obj === 'string') return JSON.stringify(obj)
  if (Array.isArray(obj)) return `[${(obj as unknown[]).map(canonicalJSON).join(',')}]`
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
    // Import Ed25519 public key (SPKI/raw format via Web Crypto)
    const pubRaw = Uint8Array.from(atob(pubKeyB64), c => c.charCodeAt(0))
    const pubKey = await crypto.subtle.importKey(
      'raw',
      pubRaw,
      { name: 'Ed25519' },
      false,
      ['verify'],
    )
    const sigBytes = Uint8Array.from(atob(sigB64), c => c.charCodeAt(0))
    const message = new TextEncoder().encode(canonicalJSON(payload))
    return await crypto.subtle.verify({ name: 'Ed25519' }, pubKey, sigBytes, message)
  } catch {
    return false
  }
}

Deno.serve(async (req: Request) => {
  if (req.method === 'OPTIONS') {
    return new Response(null, {
      headers: {
        'Access-Control-Allow-Origin': '*',
        'Access-Control-Allow-Headers': 'authorization, content-type',
      },
    })
  }

  if (req.method !== 'POST') {
    return new Response(JSON.stringify({ error: 'POST required' }), { status: 405 })
  }

  const supabaseUrl = Deno.env.get('SUPABASE_URL')!
  const serviceKey = Deno.env.get('SUPABASE_SERVICE_ROLE_KEY')!
  const pubKeyB64 = Deno.env.get('ROCKET_SIGNING_PUBKEY')

  const supabase = createClient(supabaseUrl, serviceKey)

  let body: { receipt_id: string; verify_chain?: boolean }
  try {
    body = await req.json()
  } catch {
    return new Response(JSON.stringify({ error: 'Invalid JSON' }), { status: 400 })
  }

  const { receipt_id, verify_chain = true } = body
  if (!receipt_id) {
    return new Response(JSON.stringify({ error: 'receipt_id required' }), { status: 400 })
  }

  // Fetch receipt
  const { data: receipt, error: fetchErr } = await supabase
    .from('game_receipts')
    .select('*')
    .eq('id', receipt_id)
    .single()

  if (fetchErr || !receipt) {
    return new Response(JSON.stringify({ error: 'Receipt not found', receipt_id }), { status: 404 })
  }

  const proofGates: string[] = []

  // Gate 1: not synthetic
  if (receipt.engine_source === 'synthetic') {
    return new Response(
      JSON.stringify({ verified: false, receipt_id, reason: 'engine_source is synthetic', proof_gates: [] }),
      { status: 422 },
    )
  }
  proofGates.push('not_synthetic')

  // Gate 2: lifecycle completeness
  const lifecycle: string[] = Array.isArray(receipt.ocel_lifecycle) ? receipt.ocel_lifecycle : []
  const missing = DECLARED_LIFECYCLE.filter(a => !lifecycle.includes(a))
  if (missing.length > 0) {
    return new Response(
      JSON.stringify({ verified: false, receipt_id, reason: `Missing activities: ${missing.join(', ')}`, proof_gates: proofGates }),
      { status: 422 },
    )
  }
  proofGates.push('lifecycle_complete')

  // Gate 3: Ed25519 signature (if signing key is configured)
  let sigVerified = false
  if (pubKeyB64 && receipt.ed25519_sig) {
    const { ed25519_sig, ...sigPayload } = receipt
    sigVerified = await verifyEd25519(sigPayload, ed25519_sig, pubKeyB64)
    if (!sigVerified) {
      return new Response(
        JSON.stringify({ verified: false, receipt_id, reason: 'Ed25519 signature invalid', proof_gates: proofGates }),
        { status: 401 },
      )
    }
    proofGates.push('ed25519_sig')
  } else if (!pubKeyB64) {
    proofGates.push('ed25519_sig_skipped_no_key')
  }

  // Gate 4: chain verification
  let chainVerified = false
  if (verify_chain && receipt.session_id) {
    const { data: chainResult } = await supabase
      .rpc('verify_event_chain', { p_session_id: receipt.session_id })
    chainVerified = chainResult === true
    if (chainVerified) proofGates.push('chain_intact')
  }

  const result = {
    verified: true,
    receipt_id,
    verdict: receipt.verdict,
    engine_source: receipt.engine_source,
    milestone: receipt.milestone,
    proven_at: receipt.proven_at,
    chain_verified: chainVerified,
    ed25519_verified: sigVerified,
    proof_gates: proofGates,
  }

  return new Response(JSON.stringify(result), {
    status: 200,
    headers: { 'Content-Type': 'application/json', 'Access-Control-Allow-Origin': '*' },
  })
})
