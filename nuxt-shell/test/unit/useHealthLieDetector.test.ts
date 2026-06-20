// @vitest-environment happy-dom
/**
 * useHealthLieDetector.test.ts
 *
 * Tests for the pipeline invariant detection logic (detectLies pure function).
 *
 * Invariants:
 *   LIE-1: PASS receipt with ocel_event_count=0 — evidence-free claim
 *   LIE-2: session alive >10 min with no close — stale session leak
 *   LIE-4: engine_source='synthetic' in DB — guard trigger bypass
 *
 * Van der Aalst doctrine: a pipeline that cannot detect its own lies
 * cannot be trusted. These tests are the model-vs-log comparison applied
 * to the health monitoring layer itself.
 */

import { describe, it, expect } from 'vitest'
import { detectLies, type HealthLie } from '../../server/utils/healthLies'

const NO_ROWS = null
const EMPTY = [] as Array<{ id: string }>

// ── All clear ─────────────────────────────────────────────────────────────────

describe('detectLies — all clear', () => {
  it('null inputs → no lies (DB query failed gracefully)', () => {
    const lies = detectLies(NO_ROWS, NO_ROWS, NO_ROWS)
    expect(lies).toHaveLength(0)
  })

  it('empty arrays → no lies', () => {
    const lies = detectLies(EMPTY, EMPTY, EMPTY)
    expect(lies).toHaveLength(0)
  })

  it('null mixed with empty → no lies', () => {
    expect(detectLies(NO_ROWS, EMPTY, NO_ROWS)).toHaveLength(0)
    expect(detectLies(EMPTY, NO_ROWS, EMPTY)).toHaveLength(0)
  })
})

// ── LIE-1: PASS receipt with zero events ─────────────────────────────────────

describe('detectLies — LIE-1', () => {
  it('one PASS-zero-events receipt → LIE-1 emitted', () => {
    const lies = detectLies([{ id: 'r1' }], EMPTY, EMPTY)
    expect(lies).toHaveLength(1)
    expect(lies[0]!.code).toBe('LIE-1')
  })

  it('LIE-1 description mentions count', () => {
    const lies = detectLies([{ id: 'r1' }, { id: 'r2' }], EMPTY, EMPTY)
    expect(lies[0]!.description).toContain('2')
  })

  it('LIE-1 evidence.receipts contains offending IDs', () => {
    const lies = detectLies([{ id: 'bad-receipt-1' }], EMPTY, EMPTY)
    expect((lies[0]!.evidence as { receipts: string[] }).receipts).toContain('bad-receipt-1')
  })
})

// ── LIE-2: stale alive sessions ───────────────────────────────────────────────

describe('detectLies — LIE-2', () => {
  it('one stale session → LIE-2 emitted', () => {
    const lies = detectLies(EMPTY, [{ id: 's1', project_name: 'Brm' }], EMPTY)
    expect(lies).toHaveLength(1)
    expect(lies[0]!.code).toBe('LIE-2')
  })

  it('LIE-2 evidence.sessions contains id and project', () => {
    const lies = detectLies(EMPTY, [{ id: 's1', project_name: 'ShooterGame' }], EMPTY)
    const sessions = (lies[0]!.evidence as { sessions: Array<{ id: string; project: unknown }> }).sessions
    expect(sessions[0]!.id).toBe('s1')
    expect(sessions[0]!.project).toBe('ShooterGame')
  })

  it('multiple stale sessions → all reported in one LIE-2', () => {
    const staleSessions = [
      { id: 's1', project_name: 'Brm' },
      { id: 's2', project_name: 'Brm' },
      { id: 's3', project_name: 'ShooterGame' },
    ]
    const lies = detectLies(EMPTY, staleSessions, EMPTY)
    expect(lies).toHaveLength(1)
    expect(lies[0]!.code).toBe('LIE-2')
    const sessions = (lies[0]!.evidence as { sessions: unknown[] }).sessions
    expect(sessions).toHaveLength(3)
  })
})

// ── LIE-4: synthetic engine_source in DB ─────────────────────────────────────

describe('detectLies — LIE-4', () => {
  it('one synthetic receipt → LIE-4 emitted', () => {
    const lies = detectLies(EMPTY, EMPTY, [{ id: 'syn-1' }])
    expect(lies).toHaveLength(1)
    expect(lies[0]!.code).toBe('LIE-4')
  })

  it('LIE-4 evidence.receipts contains the synthetic receipt ID', () => {
    const lies = detectLies(EMPTY, EMPTY, [{ id: 'synthetic-receipt-xyz' }])
    const receipts = (lies[0]!.evidence as { receipts: string[] }).receipts
    expect(receipts).toContain('synthetic-receipt-xyz')
  })
})

// ── Multiple simultaneous lies ────────────────────────────────────────────────

describe('detectLies — multiple lies', () => {
  it('all three violated → three lies in order LIE-1, LIE-2, LIE-4', () => {
    const lies: HealthLie[] = detectLies(
      [{ id: 'r1' }],
      [{ id: 's1', project_name: 'Brm' }],
      [{ id: 'syn-1' }],
    )
    expect(lies).toHaveLength(3)
    expect(lies.map(l => l.code)).toEqual(['LIE-1', 'LIE-2', 'LIE-4'])
  })

  it('LIE-1 + LIE-4 but no stale sessions → two lies, no LIE-2', () => {
    const lies = detectLies([{ id: 'r1' }], EMPTY, [{ id: 'syn-1' }])
    expect(lies).toHaveLength(2)
    expect(lies.map(l => l.code)).not.toContain('LIE-2')
  })
})
