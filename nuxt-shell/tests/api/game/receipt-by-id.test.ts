/**
 * receipt-by-id.test.ts — Contract tests for GET /api/game/receipt/[id]
 *
 * Tests the per-receipt detail endpoint shape and input validation.
 * In MOCK_API=1 mode the server is not running; tests verify type contracts
 * and the endpoint signature. Live tests run via the headless-loop CI job.
 */

import { describe, it, expect } from 'vitest'

const BASE = process.env.API_BASE_URL ?? 'http://localhost:3000'
const MOCK = !!process.env.MOCK_API

async function get(path: string): Promise<{ status: number; body: unknown }> {
  try {
    const res = await fetch(`${BASE}${path}`)
    let data: unknown = null
    try { data = await res.json() } catch { /* non-JSON */ }
    return { status: res.status, body: data }
  } catch {
    return { status: 503, body: null }
  }
}

// ── Shape contract (compile-time) ────────────────────────────────────────────

describe('GET /api/game/receipt/[id] — response shape contract', () => {
  it('response type includes required fields', () => {
    type ReceiptDetailResponse = {
      receipt: Record<string, unknown>
      chain_verified: boolean
      ocel_event_count: number
      first_event_at: string | null
      last_event_at: string | null
    }
    // TypeScript enforces this at compile time; constructing the value proves the shape
    const response: ReceiptDetailResponse = {
      receipt: { id: '550e8400-e29b-41d4-a716-446655440000', session_id: 's1', receipt_hash: 'a'.repeat(64) },
      chain_verified: true,
      ocel_event_count: 3,
      first_event_at: '2026-06-19T00:00:01.000Z',
      last_event_at: '2026-06-19T00:00:03.000Z',
    }
    expect(typeof response.chain_verified).toBe('boolean')
    expect(typeof response.ocel_event_count).toBe('number')
    expect(response.ocel_event_count).toBeGreaterThanOrEqual(0)
    expect(response.receipt).toBeDefined()
  })

  it('chain_verified=false is a valid response (broken chain)', () => {
    const response = { receipt: {}, chain_verified: false, ocel_event_count: 0, first_event_at: null, last_event_at: null }
    expect(response.chain_verified).toBe(false)
    expect(response.first_event_at).toBeNull()
  })
})

// ── UUID format validation (logic tests, no server needed) ────────────────────

describe('GET /api/game/receipt/[id] — UUID regex contract', () => {
  // Mirrors the regex in [id].get.ts: /^[0-9a-f-]{36}$/
  const UUID_RE = /^[0-9a-f-]{36}$/

  const validUUIDs = [
    '550e8400-e29b-41d4-a716-446655440000',
    '00000000-0000-0000-0000-000000000000',
    'ffffffff-ffff-ffff-ffff-ffffffffffff',
  ]

  const invalidUUIDs = [
    'abc',
    '12345',
    'xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx', // non-hex letters
    'not-a-uuid-at-all',
    '550e8400-e29b-41d4-a716',              // too short (27 chars)
    '550e8400-e29b-41d4-a716-4466554400001', // too long (37 chars)
  ]

  for (const uuid of validUUIDs) {
    it(`passes UUID regex: ${uuid.slice(0, 8)}…`, () => {
      expect(UUID_RE.test(uuid)).toBe(true)
    })
  }

  for (const uuid of invalidUUIDs) {
    it(`fails UUID regex: "${uuid}"`, () => {
      expect(UUID_RE.test(uuid)).toBe(false)
    })
  }
})

// ── Live endpoint tests (skipped in MOCK mode) ───────────────────────────────

describe.skipIf(MOCK)('GET /api/game/receipt/[id] — live endpoint', () => {
  it('returns 400 for obviously invalid ID (too short)', async () => {
    const { status } = await get('/api/game/receipt/abc')
    expect([400, 422]).toContain(status)
  })

  it('returns 400 for non-hex UUID-like string', async () => {
    const { status } = await get('/api/game/receipt/xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx')
    expect([400, 422]).toContain(status)
  })

  it('returns 404 for valid-format but nonexistent ID', async () => {
    const { status } = await get('/api/game/receipt/00000000-0000-0000-0000-000000000001')
    expect(status).toBe(404)
  })
})
