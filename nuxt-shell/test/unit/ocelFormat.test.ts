// @vitest-environment happy-dom
import { describe, it, expect } from 'vitest'
import * as fc from 'fast-check'
import {
  inferObjectType,
  toOcel2,
  classifyWasmCrossCheck,
  type OcelEventRow,
  type CrossCheckReceipt,
} from '../../server/utils/ocelFormat'

function makeRow(seq: number, activity: string, refs: string[] = ['session-1']): OcelEventRow {
  return {
    activity,
    timestamp_ms: 1_750_000_000_000 + seq * 1000,
    object_refs: refs,
    attributes: { stage_index: seq },
    prev_hash: seq === 0 ? null : 'a'.repeat(64),
    event_hash: 'b'.repeat(64),
    seq,
  }
}

function makeReceipt(milestone: string, verdict = 'PASS'): CrossCheckReceipt {
  return {
    id: `receipt-${milestone}`,
    milestone,
    verdict,
    engine_source: 'rocket_cli',
    output_hash: 'c'.repeat(64),
    proven_at: '2026-06-19T00:00:00Z',
    session_id: 'session-1',
  }
}

// ── inferObjectType ────────────────────────────────────────────────────────────

describe('inferObjectType', () => {
  it('session prefix → GameSession', () => expect(inferObjectType('session-abc')).toBe('GameSession'))
  it('intent prefix → Intent', () => expect(inferObjectType('intent-xyz')).toBe('Intent'))
  it('frame prefix → Frame', () => expect(inferObjectType('frame-001')).toBe('Frame'))
  it('cook prefix → CookArtifact', () => expect(inferObjectType('cook:/tmp/Brm')).toBe('CookArtifact'))
  it('unknown prefix → GameSession (fallback)', () => expect(inferObjectType('player-123')).toBe('GameSession'))
})

// ── toOcel2 ──────────────────────────────────────────────────────────────────

describe('toOcel2', () => {
  it('empty rows → valid OCEL 2.0 structure with session object', () => {
    const log = toOcel2([], 'session-1')
    expect(log.events).toHaveLength(0)
    expect(log.objects).toHaveLength(1)
    expect(log.objects[0]!.id).toBe('session-1')
    expect(log.objects[0]!.type).toBe('GameSession')
    expect(log.objectTypes).toHaveLength(1)
    expect(log.eventTypes).toHaveLength(0)
  })

  it('single event produces correct OCEL event structure', () => {
    const log = toOcel2([makeRow(0, 'GameSessionStarted')], 'session-1')
    expect(log.events).toHaveLength(1)
    const ev = log.events[0]!
    expect(ev.id).toBe('ev-0')
    expect(ev.type).toBe('GameSessionStarted')
    expect(ev.time).toMatch(/^\d{4}-\d{2}-\d{2}T/)
    // must relate to session
    expect(ev.relationships.some(r => r.objectId === 'session-1')).toBe(true)
  })

  it('events carry seq + event_hash attributes', () => {
    const log = toOcel2([makeRow(3, 'InputAdmitted')], 'session-1')
    const attrs = log.events[0]!.attributes
    expect(attrs.find(a => a.name === 'seq')?.value).toBe(3)
    expect(attrs.find(a => a.name === 'event_hash')?.value).toBe('b'.repeat(64))
  })

  it('prev_hash appears in attributes when set', () => {
    const row = makeRow(1, 'FrameRendered')
    const log = toOcel2([row], 'session-1')
    const attrs = log.events[0]!.attributes
    expect(attrs.find(a => a.name === 'prev_hash')?.value).toBe('a'.repeat(64))
  })

  it('prev_hash omitted when null (first event)', () => {
    const log = toOcel2([makeRow(0, 'GameSessionStarted')], 'session-1')
    const attrs = log.events[0]!.attributes
    expect(attrs.find(a => a.name === 'prev_hash')).toBeUndefined()
  })

  it('unique activities → unique eventTypes', () => {
    const rows = [
      makeRow(0, 'GameSessionStarted'),
      makeRow(1, 'FrameRendered'),
      makeRow(2, 'FrameRendered'), // duplicate activity
      makeRow(3, 'InputAdmitted'),
    ]
    const log = toOcel2(rows, 'session-1')
    expect(log.eventTypes).toHaveLength(3)
    const names = log.eventTypes.map(e => e.name)
    expect(names).toContain('GameSessionStarted')
    expect(names).toContain('FrameRendered')
    expect(names).toContain('InputAdmitted')
  })

  it('extra object refs appear in objects list', () => {
    const row = makeRow(0, 'IntentRegistered', ['session-1', 'intent-fire'])
    const log = toOcel2([row], 'session-1')
    const ids = log.objects.map(o => o.id)
    expect(ids).toContain('session-1')
    expect(ids).toContain('intent-fire')
    // intent prefix → Intent type
    expect(log.objects.find(o => o.id === 'intent-fire')?.type).toBe('Intent')
  })

  it('objectTypes are deduplicated across events', () => {
    const rows = [
      makeRow(0, 'A', ['session-1', 'frame-1']),
      makeRow(1, 'B', ['session-1', 'frame-2']), // same type 'Frame' for both frame refs
    ]
    const log = toOcel2(rows, 'session-1')
    const typeNames = log.objectTypes.map(t => t.name)
    const frameCount = typeNames.filter(n => n === 'Frame').length
    expect(frameCount).toBe(1) // deduplicated
  })

  // OCEL 2.0 structural invariant: every event.type must exist in eventTypes
  it('∀ event.type must appear in eventTypes (property)', () => {
    fc.assert(
      fc.property(
        fc.array(
          fc.record({
            seq: fc.nat({ max: 99 }),
            activity: fc.constantFrom('GameSessionStarted', 'FrameRendered', 'InputAdmitted', 'CookStarted'),
          }),
          { minLength: 1, maxLength: 10 },
        ),
        (rawRows) => {
          const rows = rawRows.map((r, i) => makeRow(r.seq ?? i, r.activity))
          const log = toOcel2(rows, 'session-test')
          const typeNames = new Set(log.eventTypes.map(t => t.name))
          for (const ev of log.events) {
            expect(typeNames.has(ev.type)).toBe(true)
          }
        },
      ),
    )
  })

  // OCEL 2.0 structural invariant: every relationship.objectId must exist in objects
  it('∀ relationship.objectId must appear in objects (property)', () => {
    const rows = [
      makeRow(0, 'A', ['session-1', 'intent-x', 'frame-y']),
      makeRow(1, 'B', ['session-1', 'cook:/tmp/test']),
    ]
    const log = toOcel2(rows, 'session-1')
    const objectIds = new Set(log.objects.map(o => o.id))
    for (const ev of log.events) {
      for (const rel of ev.relationships) {
        expect(objectIds.has(rel.objectId)).toBe(true)
      }
    }
  })
})

// ── classifyWasmCrossCheck ────────────────────────────────────────────────────

describe('classifyWasmCrossCheck', () => {
  it('empty receipts → NO_DATA', () => {
    expect(classifyWasmCrossCheck([]).verdict).toBe('NO_DATA')
  })

  it('cook receipt only → COOK_ONLY', () => {
    const r = classifyWasmCrossCheck([makeReceipt('HTML5CookVerify')])
    expect(r.verdict).toBe('COOK_ONLY')
    expect(r.cook_receipts).toBe(1)
    expect(r.game_receipts).toBe(0)
  })

  it('game receipt only → GAME_ONLY', () => {
    const r = classifyWasmCrossCheck([makeReceipt('GameSessionProof')])
    expect(r.verdict).toBe('GAME_ONLY')
    expect(r.game_receipts).toBe(1)
    expect(r.cook_receipts).toBe(0)
  })

  it('both cook + game → MATCH', () => {
    const r = classifyWasmCrossCheck([
      makeReceipt('HTML5CookVerify'),
      makeReceipt('GameSessionProof'),
    ])
    expect(r.verdict).toBe('MATCH')
    expect(r.cook_receipts).toBe(1)
    expect(r.game_receipts).toBe(1)
  })

  it('"HTML5PackageVerify" milestone matches cook pattern', () => {
    const r = classifyWasmCrossCheck([makeReceipt('HTML5PackageVerify')])
    expect(r.verdict).toBe('COOK_ONLY')
  })

  it('"TPS-DFLSS" milestone matches game/session pattern', () => {
    const r = classifyWasmCrossCheck([makeReceipt('TPS-DFLSS-Session')])
    expect(r.verdict).toBe('GAME_ONLY')
  })

  it('total equals receipts.length', () => {
    const receipts = [makeReceipt('CookPackage'), makeReceipt('GameSession')]
    const r = classifyWasmCrossCheck(receipts)
    expect(r.total).toBe(2)
  })

  // Property: MATCH requires at least 1 cook + 1 game
  it('∀ MATCH verdict → cook_receipts > 0 && game_receipts > 0 (property)', () => {
    fc.assert(
      fc.property(
        fc.array(
          fc.constantFrom(
            makeReceipt('HTML5CookVerify'),
            makeReceipt('GameSessionProof'),
            makeReceipt('TPS-DFLSS'),
          ),
          { minLength: 0, maxLength: 10 },
        ),
        (receipts) => {
          const r = classifyWasmCrossCheck(receipts)
          if (r.verdict === 'MATCH') {
            expect(r.cook_receipts).toBeGreaterThan(0)
            expect(r.game_receipts).toBeGreaterThan(0)
          }
        },
      ),
    )
  })
})
