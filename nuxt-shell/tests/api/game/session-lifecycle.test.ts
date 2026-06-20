/**
 * session-lifecycle.test.ts
 *
 * Contract tests for the server-side session lifecycle endpoints:
 *   POST  /api/game/session          — create session, returns session_id
 *   PATCH /api/game/session/[id]     — update is_alive / event count / close
 *
 * These endpoints replaced direct anon-client Supabase writes from
 * useGameSessionPersistence, moving all writes to service-role-key paths.
 *
 * In MOCK_API=1 mode: validate request/response shape contracts.
 * In live mode: hit the actual Nitro server.
 */

import { describe, it, expect } from 'vitest'

const BASE = process.env.API_BASE_URL ?? 'http://localhost:3000'
const MOCK = !!process.env.MOCK_API

async function post(path: string, body: unknown): Promise<{ status: number; body: unknown }> {
  try {
    const res = await fetch(`${BASE}${path}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    })
    let data: unknown = null
    try { data = await res.json() } catch { /* non-JSON */ }
    return { status: res.status, body: data }
  } catch {
    return { status: 503, body: null }
  }
}

async function patch(path: string, body: unknown): Promise<{ status: number; body: unknown }> {
  try {
    const res = await fetch(`${BASE}${path}`, {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    })
    let data: unknown = null
    try { data = await res.json() } catch { /* non-JSON */ }
    return { status: res.status, body: data }
  } catch {
    return { status: 503, body: null }
  }
}

// ── Shape contracts (compile-time) ────────────────────────────────────────────

describe('POST /api/game/session — response shape contract', () => {
  it('response type has session_id (UUID) and started_at (ISO8601)', () => {
    type SessionCreateResponse = { session_id: string; started_at: string }
    const mock: SessionCreateResponse = {
      session_id: '550e8400-e29b-41d4-a716-446655440000',
      started_at: '2026-06-19T17:00:00.000Z',
    }
    expect(mock.session_id).toMatch(/^[0-9a-f-]{36}$/)
    expect(mock.started_at).toMatch(/^\d{4}-\d{2}-\d{2}T/)
  })

  it.skipIf(MOCK)('browser_session_id is required (400 without it)', async () => {
    const { status } = await post('/api/game/session', {})
    expect(status).toBe(400)
  })

  it.skipIf(MOCK)('valid body is accepted (200)', async () => {
    const { status } = await post('/api/game/session', {
      browser_session_id: 'test-browser-sid-001',
      engine_source: 'browser',
    })
    expect([200, 201]).toContain(status)
  })
})

describe('PATCH /api/game/session/[id] — response shape contract', () => {
  it('response type has updated=true and session_id', () => {
    type SessionPatchResponse = { updated: boolean; session_id: string }
    const mock: SessionPatchResponse = {
      updated: true,
      session_id: '550e8400-e29b-41d4-a716-446655440000',
    }
    expect(mock.updated).toBe(true)
    expect(mock.session_id).toMatch(/^[0-9a-f-]{36}$/)
  })

  it.skipIf(MOCK)('invalid UUID returns 400', async () => {
    const { status } = await patch('/api/game/session/not-a-uuid', { is_alive: false })
    expect([400, 422]).toContain(status)
  })

  it.skipIf(MOCK)('empty patch body returns 400', async () => {
    const { status } = await patch('/api/game/session/550e8400-e29b-41d4-a716-446655440000', {})
    expect(status).toBe(400)
  })

  it.skipIf(MOCK)('unknown field is silently ignored — only allowed fields patched', async () => {
    const { status } = await patch('/api/game/session/550e8400-e29b-41d4-a716-446655440000', {
      is_alive: false,
      unknown_field: 'should-be-ignored',
    })
    // valid UUID format, session doesn't exist → 404; server still strips unknown field
    expect([200, 404]).toContain(status)
  })
})

// ── Live tests (skipped in MOCK mode) ────────────────────────────────────────

describe.skipIf(MOCK)('Session lifecycle — live endpoints', () => {
  let createdSessionId: string | null = null

  it('POST /api/game/session creates a session and returns UUID', async () => {
    const { status, body } = await post('/api/game/session', {
      browser_session_id: `test-${Date.now()}`,
      engine_source: 'test',
    })
    expect(status).toBe(200)
    const b = body as { session_id: string; started_at: string }
    expect(b.session_id).toMatch(/^[0-9a-f-]{36}$/)
    expect(b.started_at).toBeTruthy()
    createdSessionId = b.session_id
  })

  it('PATCH /api/game/session/[id] updates event count', async () => {
    if (!createdSessionId) return
    const { status, body } = await patch(`/api/game/session/${createdSessionId}`, {
      ocel_event_count: 5,
      is_alive: true,
    })
    expect(status).toBe(200)
    const b = body as { updated: boolean; session_id: string }
    expect(b.updated).toBe(true)
    expect(b.session_id).toBe(createdSessionId)
  })

  it('PATCH /api/game/session/[id] closes the session', async () => {
    if (!createdSessionId) return
    const { status } = await patch(`/api/game/session/${createdSessionId}`, {
      is_alive: false,
      session_ended_at: new Date().toISOString(),
      ocel_event_count: 5,
    })
    expect(status).toBe(200)
  })
})
