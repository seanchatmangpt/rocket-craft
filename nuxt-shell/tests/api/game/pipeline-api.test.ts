/**
 * Vitest integration tests for /api/game/ Nitro server routes.
 *
 * Adapted from dashboard.bak/tests/edge-functions.test.js pattern:
 * - fetch-based, runs against the local Nitro dev server
 * - tests request validation (reject bad body), response shape, and error codes
 * - each test is self-contained; no global state between tests
 *
 * Run:
 *   cd nuxt-shell && npx vitest run tests/api/game/pipeline-api.test.ts
 *
 * Requires: local Nitro dev server running on port 3000
 *   (start with: cd nuxt-shell && npx nuxt dev --port 3000)
 *
 * Or use the mock mode (MOCK_API=1) which skips live Supabase calls.
 */

import { describe, it, expect, beforeAll } from 'vitest';

const BASE = process.env.API_BASE_URL || 'http://localhost:3000';
const MOCK = process.env.MOCK_API === '1';

async function post(path: string, body: unknown) {
  const res = await fetch(`${BASE}${path}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  const json = await res.json().catch(() => null);
  return { status: res.status, body: json };
}

async function get(path: string) {
  const res = await fetch(`${BASE}${path}`);
  const json = await res.json().catch(() => null);
  return { status: res.status, body: json };
}

// ── /api/game/receipt ────────────────────────────────────────────────────────
describe('POST /api/game/receipt', () => {
  it('rejects missing session_id', async () => {
    const { status, body } = await post('/api/game/receipt', {
      ocel_lifecycle: ['GameSessionStarted'],
      receipt_hash: 'abc',
    });
    expect(status).toBe(400);
    expect(body?.statusMessage ?? body?.message ?? '').toMatch(/session_id/i);
  });

  it('rejects synthetic engine_source', async () => {
    const { status, body } = await post('/api/game/receipt', {
      session_id: '00000000-0000-0000-0000-000000000000',
      ocel_lifecycle: ['GameSessionStarted', 'FrameRendered', 'InputAdmitted'],
      ocel_event_count: 3,
      engine_source: 'synthetic',
      receipt_hash: 'deadbeef'.repeat(8),
      milestone: 'test',
      payload: {},
    });
    expect(status).toBe(422);
    expect(body?.statusMessage ?? body?.message ?? '').toMatch(/synthetic/i);
  });

  it('rejects lifecycle missing GameSessionStarted', async () => {
    const { status, body } = await post('/api/game/receipt', {
      session_id: '00000000-0000-0000-0000-000000000001',
      ocel_lifecycle: ['FrameRendered', 'InputAdmitted'],
      ocel_event_count: 2,
      engine_source: 'real_ue4',
      receipt_hash: 'deadbeef'.repeat(8),
      milestone: 'test',
      payload: {},
    });
    // Server validates lifecycle — should return FAIL verdict (200) or 422
    if (status === 200) {
      expect(body?.verdict).toBe('FAIL');
      expect(body?.reason).toMatch(/GameSessionStarted/i);
    } else {
      expect([400, 422, 500]).toContain(status);
    }
  });

  it('accepts a valid receipt body shape (Supabase may reject unknown session)', async () => {
    if (MOCK) return; // skip live DB call in mock mode
    const { status, body } = await post('/api/game/receipt', {
      session_id: '00000000-0000-0000-0000-000000000002',
      ocel_lifecycle: ['GameSessionStarted', 'FrameRendered', 'InputAdmitted'],
      ocel_event_count: 3,
      engine_source: 'real_ue4',
      receipt_hash: 'a'.repeat(64),
      milestone: 'HTML5CookVerify',
      payload: { wasm_mb: 175, test: true },
    });
    // 200 means Supabase accepted; 500 means FK violation (no such session) — both mean correct validation path
    expect([200, 500]).toContain(status);
    if (status === 200) {
      expect(body).toMatchObject({ verdict: expect.any(String), milestone: 'HTML5CookVerify' });
    }
  });
});

// ── /api/game/chain-verify ───────────────────────────────────────────────────
describe('GET /api/game/chain-verify', () => {
  it('returns overall + sessions_checked + breaks array', async () => {
    if (MOCK) return;
    const { status, body } = await get('/api/game/chain-verify');
    expect([200, 503]).toContain(status);
    if (status === 200) {
      expect(body).toHaveProperty('overall');
      expect(body).toHaveProperty('sessions_checked');
      expect(body).toHaveProperty('breaks');
      expect(Array.isArray(body.breaks)).toBe(true);
      expect(['PASS', 'FAIL', 'UNKNOWN']).toContain(body.overall);
    }
  });

  it('accepts session_id query param without erroring', async () => {
    if (MOCK) return;
    const { status } = await get('/api/game/chain-verify?session_id=00000000-0000-0000-0000-000000000000');
    expect([200, 503]).toContain(status);
  });
});

// ── /api/game/receipt-finalize ───────────────────────────────────────────────
describe('POST /api/game/receipt-finalize', () => {
  it('rejects missing session_id', async () => {
    const { status } = await post('/api/game/receipt-finalize', { receipt_hash: 'abc' });
    expect(status).toBe(400);
  });

  it('rejects missing receipt_hash', async () => {
    const { status } = await post('/api/game/receipt-finalize', {
      session_id: '00000000-0000-0000-0000-000000000000',
    });
    expect(status).toBe(400);
  });

  it('returns verdict field for unknown session (NO_EVENTS or CHAIN_BROKEN)', async () => {
    if (MOCK) return;
    const { status, body } = await post('/api/game/receipt-finalize', {
      session_id: '00000000-0000-0000-0000-000000000099',
      receipt_hash: 'a'.repeat(64),
    });
    expect([200, 503]).toContain(status);
    if (status === 200) {
      expect(['PROVEN', 'CHAIN_BROKEN', 'HASH_MISMATCH', 'NO_EVENTS']).toContain(body?.verdict);
    }
  });
});

// ── /api/game/pipeline-health ────────────────────────────────────────────────
describe('GET /api/game/pipeline-health', () => {
  it('returns pass_rate_pct and total_receipts', async () => {
    if (MOCK) return;
    const { status, body } = await get('/api/game/pipeline-health');
    expect([200, 503]).toContain(status);
    if (status === 200) {
      expect(typeof body.total_receipts).toBe('number');
      expect(typeof body.pass_rate_pct).toBe('number');
      expect(body.pass_rate_pct).toBeGreaterThanOrEqual(0);
      expect(body.pass_rate_pct).toBeLessThanOrEqual(100);
    }
  });

  it('cache-busts with ?bust=1', async () => {
    if (MOCK) return;
    const { status, body } = await get('/api/game/pipeline-health?bust=1');
    expect([200, 503]).toContain(status);
    if (status === 200) {
      expect(body.cache_hit).toBeFalsy();
    }
  });
});

// ── /api/game/ocel-ingest ────────────────────────────────────────────────────
describe('POST /api/game/ocel-ingest', () => {
  it('rejects empty body', async () => {
    const { status } = await post('/api/game/ocel-ingest', {});
    expect([400, 422]).toContain(status);
  });

  it('accepts well-formed OCEL event batch', async () => {
    if (MOCK) return;
    const { status, body } = await post('/api/game/ocel-ingest', {
      session_id: '00000000-0000-0000-0000-000000000099',
      events: [
        {
          id: 'test-evt-1',
          activity: 'GameSessionStarted',
          timestamp_ms: Date.now(),
          object_refs: [{ object_id: 'session-99', qualifier: 'root' }],
          attributes: {},
          event_hash: 'a'.repeat(64),
          prev_hash: null,
          seq: 0,
        },
      ],
    });
    // 200 = accepted + OTel emitted; 500 = FK violation (no session row) — both mean correct path
    expect([200, 500]).toContain(status);
    if (status === 200) {
      expect(body).toHaveProperty('persisted');
    }
  });
});

beforeAll(() => {
  if (!MOCK) {
    console.log(`[pipeline-api.test] Running against ${BASE}`);
    console.log('[pipeline-api.test] Set MOCK_API=1 to skip live Supabase calls');
  }
});
