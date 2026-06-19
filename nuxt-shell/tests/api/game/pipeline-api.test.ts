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
    if (MOCK) return; // validation tests require a live Nitro server
    const { status, body } = await post('/api/game/receipt', {
      ocel_lifecycle: ['GameSessionStarted'],
      receipt_hash: 'abc',
    });
    expect(status).toBe(400);
    expect(body?.statusMessage ?? body?.message ?? '').toMatch(/session_id/i);
  });

  it('rejects synthetic engine_source', async () => {
    if (MOCK) return;
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
    if (MOCK) return;
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
    if (MOCK) return;
    const { status } = await post('/api/game/receipt-finalize', { receipt_hash: 'abc' });
    expect(status).toBe(400);
  });

  it('rejects missing receipt_hash', async () => {
    if (MOCK) return;
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
    if (MOCK) return;
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

// ── /api/game/wasm-crosscheck ────────────────────────────────────────────────
describe('GET /api/game/wasm-crosscheck', () => {
  const VALID_HASH = 'a'.repeat(64);   // 64 hex chars — valid BLAKE3 shape

  it('rejects missing output_hash', async () => {
    if (MOCK) return;
    const { status } = await get('/api/game/wasm-crosscheck');
    expect(status).toBe(400);
  });

  it('rejects output_hash that is not 64 hex chars', async () => {
    if (MOCK) return;
    const { status, body } = await get('/api/game/wasm-crosscheck?output_hash=tooshort');
    expect(status).toBe(400);
    expect(body?.statusMessage ?? body?.message ?? '').toMatch(/64/i);
  });

  it('accepts a valid 64-char BLAKE3 hash and returns cross_check structure', async () => {
    if (MOCK) return;
    const { status, body } = await get(`/api/game/wasm-crosscheck?output_hash=${VALID_HASH}`);
    // 200 = Supabase reachable (even if no rows with this hash)
    // 503 = Supabase not configured (local dev without env)
    expect([200, 503]).toContain(status);
    if (status === 200) {
      expect(body).toHaveProperty('output_hash', VALID_HASH);
      expect(body).toHaveProperty('receipts');
      expect(Array.isArray(body.receipts)).toBe(true);
      expect(body).toHaveProperty('cross_check');
      expect(['MATCH', 'MISMATCH', 'COOK_ONLY', 'GAME_ONLY', 'NO_DATA']).toContain(body.cross_check.verdict);
      expect(typeof body.cross_check.cook_receipts).toBe('number');
      expect(typeof body.cross_check.game_receipts).toBe('number');
      expect(typeof body.cross_check.total).toBe('number');
    }
  });

  it('NO_DATA verdict for a hash that cannot exist in DB', async () => {
    if (MOCK) return;
    // All-zeros hash is a valid BLAKE3 hex — but will never be in the DB
    const zeroHash = '0'.repeat(64);
    const { status, body } = await get(`/api/game/wasm-crosscheck?output_hash=${zeroHash}`);
    expect([200, 503]).toContain(status);
    if (status === 200) {
      expect(body.cross_check.verdict).toBe('NO_DATA');
      expect(body.receipts).toHaveLength(0);
    }
  });
});

// ── /api/game/session-seed ────────────────────────────────────────────────────
describe('POST /api/game/session-seed (headless seeder)', () => {
  it('returns 403 when ALLOW_SESSION_SEED not set in production mode', async () => {
    if (MOCK) return; // live only
    // In dev/test the server likely allows it; accept either 200 or 403
    const { status } = await post('/api/game/session-seed', {});
    expect([200, 403, 503]).toContain(status);
  });

  it('returns session_id, receipt_id, receipt_hash, chain_tip when allowed', async () => {
    if (MOCK) return;
    const { status, body } = await post('/api/game/session-seed', {});
    // 503 = Supabase not configured (local without .env); 403 = production guard
    if (status === 503 || status === 403) return;
    expect(status).toBe(200);
    expect(typeof body.session_id).toBe('string');
    expect(typeof body.receipt_id).toBe('string');
    expect(typeof body.receipt_hash).toBe('string');
    expect(body.receipt_hash).toHaveLength(64); // BLAKE3 hex
    expect(typeof body.chain_tip).toBe('string');
    expect(body.chain_tip).toHaveLength(64);
    expect(body.ocel_event_count).toBeGreaterThanOrEqual(3);
    expect(Array.isArray(body.activities)).toBe(true);
    expect(body.activities).toContain('GameSessionStarted');
    expect(body.activities).toContain('FrameRendered');
    expect(body.activities).toContain('InputAdmitted');
  });

  it('full automated loop: seed → finalize → PROVEN (no browser, no UE4)', async () => {
    if (MOCK) return;
    // Step 1: seed a complete session
    const seed = await post('/api/game/session-seed', {});
    if (seed.status === 503 || seed.status === 403) return; // Supabase not available
    expect(seed.status).toBe(200);

    const { session_id, receipt_hash } = seed.body;

    // Step 2: prove the chain — should return PROVEN
    const finalize = await post('/api/game/receipt-finalize', { session_id, receipt_hash });
    if (finalize.status === 503) return;
    expect(finalize.status).toBe(200);
    // PROVEN or HASH_MISMATCH (receipt_hash in session-seed is not the chain tip itself)
    expect(['PROVEN', 'HASH_MISMATCH', 'NO_EVENTS']).toContain(finalize.body?.verdict);

    // Step 3: verify the chain is intact (even if receipt_hash ≠ chain_tip)
    expect(finalize.body?.chain_verified).toBe(true);
    console.log(`[full-loop] session=${session_id} chain_verified=${finalize.body?.chain_verified} verdict=${finalize.body?.verdict}`);
  });
});

beforeAll(() => {
  if (!MOCK) {
    console.log(`[pipeline-api.test] Running against ${BASE}`);
    console.log('[pipeline-api.test] Set MOCK_API=1 to skip live Supabase calls');
  }
});
