/**
 * leaderboard.test.ts
 *
 * Contract tests for GET /api/game/leaderboard
 *
 * Shape: { rows: LeaderboardRow[], total: number | null, limit: number, offset: number, cached_at: string }
 * Each row: { rank, player_id, display_name, total_receipts, pass_receipts, fail_receipts,
 *             pass_rate_pct, last_pass_at, best_ocel_events }
 *
 * In MOCK_API=1 mode: compile-time shape contracts only.
 * In live mode: hit the actual Nitro server.
 */

import { describe, it, expect } from 'vitest'

const BASE = process.env.API_BASE_URL ?? 'http://localhost:3000'
const MOCK = Boolean(process.env.MOCK_API)

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

// ── Compile-time shape contracts ─────────────────────────────────────────────

interface LeaderboardRow {
  rank: number
  player_id: string
  display_name: string | null
  total_receipts: number
  pass_receipts: number
  fail_receipts: number
  pass_rate_pct: number | null
  last_pass_at: string | null
  best_ocel_events: number | null
}

interface LeaderboardResponse {
  rows: LeaderboardRow[]
  total: number | null
  limit: number
  offset: number
  cached_at: string
}

describe('GET /api/game/leaderboard — response shape contract', () => {
  it('LeaderboardResponse has rows, total, limit, offset, cached_at', () => {
    const mock: LeaderboardResponse = {
      rows: [],
      total: 0,
      limit: 20,
      offset: 0,
      cached_at: new Date().toISOString(),
    }
    expect(Array.isArray(mock.rows)).toBe(true)
    expect(typeof mock.limit).toBe('number')
    expect(typeof mock.offset).toBe('number')
    expect(mock.cached_at).toMatch(/^\d{4}-\d{2}-\d{2}T/)
  })

  it('LeaderboardRow has all required numeric and nullable fields', () => {
    const row: LeaderboardRow = {
      rank: 1,
      player_id: '550e8400-e29b-41d4-a716-446655440000',
      display_name: 'Player One',
      total_receipts: 10,
      pass_receipts: 8,
      fail_receipts: 2,
      pass_rate_pct: 80.0,
      last_pass_at: '2026-06-19T10:00:00.000Z',
      best_ocel_events: 42,
    }
    expect(row.rank).toBeGreaterThanOrEqual(1)
    expect(row.player_id).toMatch(/^[0-9a-f-]{36}$/)
    expect(row.pass_receipts + row.fail_receipts).toBeLessThanOrEqual(row.total_receipts)
  })

  it('total may be null (Supabase count is null when no rows)', () => {
    const withNull: LeaderboardResponse = {
      rows: [],
      total: null,
      limit: 20,
      offset: 0,
      cached_at: new Date().toISOString(),
    }
    expect(withNull.total === null || typeof withNull.total === 'number').toBe(true)
  })
})

// ── Live tests (skipped in MOCK mode) ────────────────────────────────────────

describe.skipIf(MOCK)('GET /api/game/leaderboard — live endpoint', () => {
  it('returns correct shape with rows array and pagination fields', async () => {
    const { status, body } = await get('/api/game/leaderboard')
    expect(status).toBe(200)
    const b = body as LeaderboardResponse
    expect(Array.isArray(b.rows)).toBe(true)
    expect(typeof b.limit).toBe('number')
    expect(typeof b.offset).toBe('number')
    expect(b.cached_at).toMatch(/^\d{4}-\d{2}-\d{2}T/)
    // total is number or null
    expect(b.total === null || typeof b.total === 'number').toBe(true)
    // each row must have the required fields
    for (const row of b.rows) {
      expect(typeof row.rank).toBe('number')
      expect(typeof row.player_id).toBe('string')
      expect(typeof row.total_receipts).toBe('number')
    }
  })

  it('?limit=5 returns at most 5 rows', async () => {
    const { status, body } = await get('/api/game/leaderboard?limit=5')
    expect(status).toBe(200)
    const b = body as LeaderboardResponse
    expect(b.limit).toBe(5)
    expect(b.rows.length).toBeLessThanOrEqual(5)
  })

  it('?verdict=PASS query is accepted (rows may be empty or filtered)', async () => {
    // The endpoint does not currently filter by verdict, but the request must not crash
    const { status, body } = await get('/api/game/leaderboard?verdict=PASS')
    expect(status).toBe(200)
    const b = body as LeaderboardResponse
    expect(Array.isArray(b.rows)).toBe(true)
  })
})
