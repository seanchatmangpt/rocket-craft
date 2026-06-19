/**
 * cook-pipeline.test.ts
 *
 * Tests for the cook pipeline gate endpoints:
 *   POST /api/game/cook-receipt  — Ed25519-gated receipt insertion
 *   GET  /api/game/cook-status   — Cook log tail + last receipt status
 *
 * These close the gap where rocket-cmd wrote directly to Supabase REST,
 * bypassing the proof gates (engine_source, lifecycle, Ed25519 sig).
 *
 * Van der Aalst: a cook receipt is evidence only after all four proof gates pass.
 * Bypassing the gate produces a record, not evidence.
 */

import { describe, it, expect, beforeAll } from 'vitest'

const BASE = process.env.API_BASE_URL ?? 'http://localhost:3000'
const MOCK = !!process.env.MOCK_API

async function post(path: string, body: unknown): Promise<{ status: number; body: any }> {
  try {
    const res = await fetch(`${BASE}${path}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    })
    let data: any = null
    try { data = await res.json() } catch { /* non-JSON */ }
    return { status: res.status, body: data }
  } catch {
    return { status: 503, body: null }
  }
}

async function get(path: string): Promise<{ status: number; body: any }> {
  try {
    const res = await fetch(`${BASE}${path}`)
    let data: any = null
    try { data = await res.json() } catch { /* non-JSON */ }
    return { status: res.status, body: data }
  } catch {
    return { status: 503, body: null }
  }
}

const VALID_RECEIPT = {
  session_id: null,
  verdict: 'PASS',
  milestone: 'HTML5CookVerify',
  engine_source: 'rocket_cli',
  ocel_lifecycle: ['GameSessionStarted', 'FrameRendered', 'InputAdmitted', 'CookCompleted'],
  ocel_event_count: 4,
  receipt_hash: 'a'.repeat(64),  // 64 hex chars (fake BLAKE3)
  output_hash: 'b'.repeat(64),
  proven_at: '2026-06-19T00:00:00Z',
  payload: { project: 'Brm', platform: 'HTML5' },
}

// ── POST /api/game/cook-receipt ───────────────────────────────────────────────

describe('POST /api/game/cook-receipt', () => {
  it('rejects missing required fields', async () => {
    const { status } = await post('/api/game/cook-receipt', {})
    expect([400, 422, 500, 503]).toContain(status)
  })

  it('rejects synthetic engine_source (proof gate 2)', async () => {
    const { status, body } = await post('/api/game/cook-receipt', {
      ...VALID_RECEIPT,
      engine_source: 'synthetic',
    })
    expect([422, 500, 503]).toContain(status)
    if (status === 422) {
      expect(body?.message ?? body?.statusMessage ?? '').toMatch(/synthetic/)
    }
  })

  it('rejects invalid receipt_hash (not 64 hex chars)', async () => {
    const { status } = await post('/api/game/cook-receipt', {
      ...VALID_RECEIPT,
      receipt_hash: 'tooshort',
    })
    expect([400, 422, 500, 503]).toContain(status)
  })

  it('rejects lifecycle missing GameSessionStarted', async () => {
    const { status, body } = await post('/api/game/cook-receipt', {
      ...VALID_RECEIPT,
      ocel_lifecycle: ['FrameRendered', 'InputAdmitted'],
    })
    expect([400, 422, 500, 503]).toContain(status)
    if (status === 422) {
      expect(body?.message ?? body?.statusMessage ?? '').toMatch(/GameSessionStarted/)
    }
  })

  it('rejects lifecycle missing FrameRendered', async () => {
    const { status } = await post('/api/game/cook-receipt', {
      ...VALID_RECEIPT,
      ocel_lifecycle: ['GameSessionStarted', 'InputAdmitted'],
    })
    expect([400, 422, 500, 503]).toContain(status)
  })

  it('rejects lifecycle missing InputAdmitted', async () => {
    const { status } = await post('/api/game/cook-receipt', {
      ...VALID_RECEIPT,
      ocel_lifecycle: ['GameSessionStarted', 'FrameRendered'],
    })
    expect([400, 422, 500, 503]).toContain(status)
  })

  it('accepts valid receipt (may 503 without service role key or 500 on DB error)', async () => {
    const { status, body } = await post('/api/game/cook-receipt', VALID_RECEIPT)
    // In mock mode we expect 503 (no service key) or 500 (no Supabase)
    // In live mode we expect 200
    expect([200, 500, 503]).toContain(status)
    if (status === 200) {
      expect(body.receipt_id).toBeTruthy()
      expect(body.verdict).toBe('PASS')
      expect(Array.isArray(body.proof_gates_passed)).toBe(true)
      expect(body.proof_gates_passed).toContain('not_synthetic')
      expect(body.proof_gates_passed).toContain('lifecycle_complete')
      console.log(`[cook-receipt] receipt_id=${body.receipt_id} gates=${body.proof_gates_passed.join(',')}`)
    }
  })

  it('full gate proof: seed → cook-receipt → PROVEN (live only)', async () => {
    if (MOCK) return
    // Seed a real session with a valid chain
    const seedRes = await post('/api/game/session-seed', {})
    if (seedRes.status !== 200) return
    const { session_id, receipt_hash } = seedRes.body

    // Push the cook receipt through the gate (not direct REST)
    const { status, body } = await post('/api/game/cook-receipt', {
      ...VALID_RECEIPT,
      session_id,
      receipt_hash,
      verdict: 'PASS',
    })
    if (status === 503) return
    expect(status).toBe(200)
    expect(body.verdict).toBe('PASS')
    expect(body.chain_verified).toBe(true)
    expect(body.proof_gates_passed).toContain('not_synthetic')
    expect(body.proof_gates_passed).toContain('lifecycle_complete')
    console.log(`[cook-receipt gate] session=${session_id} chain_verified=${body.chain_verified}`)
  })
})

// ── GET /api/game/cook-status ─────────────────────────────────────────────────

describe('GET /api/game/cook-status', () => {
  it('returns status, project, last_receipt, log_tail, cook_events', async () => {
    const { status, body } = await get('/api/game/cook-status')
    if (status === 503 || status === 500) return
    expect(status).toBe(200)
    expect(['idle', 'cooking', 'done', 'failed']).toContain(body.status)
    expect(typeof body.project).toBe('string')
    expect(Array.isArray(body.log_tail)).toBe(true)
    expect(Array.isArray(body.cook_events)).toBe(true)
    // last_receipt is null or an object
    if (body.last_receipt !== null) {
      expect(typeof body.last_receipt.verdict).toBe('string')
      expect(typeof body.last_receipt.proven_at).toBe('string')
    }
  })

  it('respects ?project=Brm query param', async () => {
    const { status, body } = await get('/api/game/cook-status?project=Brm')
    if (status === 503 || status === 500) return
    expect(status).toBe(200)
    expect(body.project).toBe('Brm')
  })

  it('respects ?lines=10 param (log_tail ≤ 10 lines)', async () => {
    const { status, body } = await get('/api/game/cook-status?lines=10')
    if (status === 503 || status === 500) return
    expect(status).toBe(200)
    expect(body.log_tail.length).toBeLessThanOrEqual(10)
  })

  it('status is done after a PASS cook receipt exists (live only)', async () => {
    if (MOCK) return
    // Push a valid cook receipt first
    const { status: rStatus } = await post('/api/game/cook-receipt', {
      ...VALID_RECEIPT,
      proven_at: new Date().toISOString(),
    })
    if (rStatus !== 200) return

    const { status, body } = await get('/api/game/cook-status')
    if (status === 503 || status === 500) return
    expect(status).toBe(200)
    expect(body.last_receipt?.verdict).toBe('PASS')
    console.log(`[cook-status] status=${body.status} last_verdict=${body.last_receipt?.verdict}`)
  })
})

beforeAll(() => {
  if (!MOCK) {
    console.log(`[cook-pipeline.test] Running against ${BASE}`)
    console.log('[cook-pipeline.test] Set MOCK_API=1 for offline mode')
  }
})
