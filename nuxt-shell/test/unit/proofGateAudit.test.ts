// @vitest-environment happy-dom
/**
 * proofGateAudit.test.ts
 *
 * Tests for the proof gate audit utility — specifically the gate-name mapping
 * and the audit entry shape. The fire-and-forget insert is not tested here
 * (that's an integration concern); we test the pure logic of what gets recorded.
 */

import { describe, it, expect, vi } from 'vitest'
import { auditGateRun, recordGateAudit } from '../../server/utils/proofGateAudit'

function makeSb(insertFn = vi.fn().mockResolvedValue({ error: null })) {
  return {
    from: () => ({ insert: insertFn }),
  } as unknown as Parameters<typeof recordGateAudit>[0]
}

// ── recordGateAudit ──────────────────────────────────────────────────────────

describe('recordGateAudit', () => {
  it('calls sb.from("proof_gate_audits").insert with correct fields', async () => {
    const insertFn = vi.fn().mockResolvedValue({ error: null })
    const sb = makeSb(insertFn)

    await recordGateAudit(sb, {
      session_id: 'sess-1',
      gate_name: 'not_synthetic',
      outcome: 'fail',
      input_summary: 'synthetic',
      reason: 'engine_source: synthetic is rejected',
      evaluated_at: '2026-01-01T00:00:00.000Z',
    })

    expect(insertFn).toHaveBeenCalledOnce()
    const arg = insertFn.mock.calls[0][0]
    expect(arg.gate_name).toBe('not_synthetic')
    expect(arg.outcome).toBe('fail')
    expect(arg.reason).toContain('synthetic')
  })

  it('swallows insert errors (fire-and-forget)', async () => {
    const sb = makeSb(vi.fn().mockRejectedValue(new Error('DB down')))
    // Must not throw
    await expect(recordGateAudit(sb, {
      session_id: 's1',
      gate_name: 'required_fields',
      outcome: 'pass',
      input_summary: '{}',
      reason: null,
      evaluated_at: new Date().toISOString(),
    })).resolves.toBeUndefined()
  })
})

// ── auditGateRun — pass path ─────────────────────────────────────────────────

describe('auditGateRun — all gates pass', () => {
  it('records 4 pass entries for a clean input', () => {
    const insertCalls: unknown[] = []
    const sb = {
      from: () => ({ insert: (row: unknown) => { insertCalls.push(row); return Promise.resolve({ error: null }) } }),
    } as unknown as Parameters<typeof auditGateRun>[0]

    auditGateRun(
      sb,
      'sess-ok',
      {
        verdict: 'PASS',
        milestone: 'HeadlessSeed',
        engine_source: 'rocket_cli',
        receipt_hash: 'a'.repeat(64),
        ocel_lifecycle: ['GameSessionStarted', 'FrameRendered', 'InputAdmitted'],
      },
      null,
      null,
    )

    // All 4 gates should be recorded as 'pass' (fire-and-forget; entries queued async)
    expect(insertCalls).toHaveLength(4)
    for (const entry of insertCalls as Array<{ outcome: string }>) {
      expect(entry.outcome).toBe('pass')
    }
  })
})

// ── auditGateRun — fail path ─────────────────────────────────────────────────

describe('auditGateRun — gate fails', () => {
  it('records fail for the failing gate and skips subsequent gates', () => {
    const insertCalls: unknown[] = []
    const sb = {
      from: () => ({ insert: (row: unknown) => { insertCalls.push(row); return Promise.resolve({ error: null }) } }),
    } as unknown as Parameters<typeof auditGateRun>[0]

    auditGateRun(
      sb,
      'sess-bad',
      {
        verdict: 'PASS',
        milestone: 'HeadlessSeed',
        engine_source: 'synthetic', // triggers not_synthetic gate
        receipt_hash: 'a'.repeat(64),
        ocel_lifecycle: ['GameSessionStarted', 'FrameRendered', 'InputAdmitted'],
      },
      'not_synthetic',
      'engine_source: synthetic is rejected by the proof gate',
    )

    // Gates before and including the fail gate are recorded; subsequent gates skipped
    const entries = insertCalls as Array<{ gate_name: string; outcome: string }>
    const failEntry = entries.find(e => e.gate_name === 'not_synthetic')
    expect(failEntry).toBeDefined()
    expect(failEntry!.outcome).toBe('fail')

    // No entries after the fail gate (lifecycle_complete should be skipped)
    const afterFail = entries.find(e => e.gate_name === 'lifecycle_complete')
    expect(afterFail).toBeUndefined()
  })

  it('records reason on the fail entry', () => {
    const insertCalls: unknown[] = []
    const sb = {
      from: () => ({ insert: (row: unknown) => { insertCalls.push(row); return Promise.resolve({ error: null }) } }),
    } as unknown as Parameters<typeof auditGateRun>[0]

    auditGateRun(sb, 's', { verdict: 'PASS', milestone: 'M', engine_source: 'synthetic', receipt_hash: 'x', ocel_lifecycle: [] }, 'not_synthetic', 'rejected')
    const fail = (insertCalls as Array<{ gate_name: string; reason: string }>).find(e => e.gate_name === 'not_synthetic')
    expect(fail?.reason).toBe('rejected')
  })
})
