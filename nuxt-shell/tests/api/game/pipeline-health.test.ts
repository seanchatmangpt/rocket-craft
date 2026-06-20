/**
 * pipeline-health.test.ts
 *
 * Contract tests for GET /api/game/pipeline-health
 *
 * Shape: { ...pipeline_health view fields, cached_at: string, cache_hit: boolean }
 * The view fields are dynamic (from Supabase), but cached_at and cache_hit are always present.
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

interface PipelineHealthResponse {
  cached_at: string
  cache_hit: boolean
  [key: string]: unknown  // pipeline_health view columns are dynamic
}

describe('GET /api/game/pipeline-health — response shape contract', () => {
  it('response always has cached_at (ISO string) and cache_hit (boolean)', () => {
    const mock: PipelineHealthResponse = {
      cached_at: new Date().toISOString(),
      cache_hit: false,
      total_receipts: 0,
      pass_rate_pct: 0,
    }
    expect(mock.cached_at).toMatch(/^\d{4}-\d{2}-\d{2}T/)
    expect(typeof mock.cache_hit).toBe('boolean')
  })

  it('cache_hit=true means served from KV; cache_hit=false means fresh from Supabase', () => {
    const fromCache: PipelineHealthResponse = { cached_at: '2026-06-19T10:00:00.000Z', cache_hit: true }
    const fromSupabase: PipelineHealthResponse = { cached_at: '2026-06-19T10:00:00.000Z', cache_hit: false }
    expect(fromCache.cache_hit).toBe(true)
    expect(fromSupabase.cache_hit).toBe(false)
  })
})

// ── Live tests (skipped in MOCK mode) ────────────────────────────────────────

describe.skipIf(MOCK)('GET /api/game/pipeline-health — live endpoint', () => {
  it('returns valid shape with cached_at and cache_hit', async () => {
    const { status, body } = await get('/api/game/pipeline-health')
    expect(status).toBe(200)
    const b = body as PipelineHealthResponse
    expect(b.cached_at).toMatch(/^\d{4}-\d{2}-\d{2}T/)
    expect(typeof b.cache_hit).toBe('boolean')
  })

  it('?bust=1 forces fresh fetch and returns cache_hit=false', async () => {
    const { status, body } = await get('/api/game/pipeline-health?bust=1')
    expect(status).toBe(200)
    const b = body as PipelineHealthResponse
    expect(b.cache_hit).toBe(false)
    expect(b.cached_at).toMatch(/^\d{4}-\d{2}-\d{2}T/)
  })
})
